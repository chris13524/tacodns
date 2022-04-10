use anyhow::{anyhow, Result};
use log::*;
use reqwest::Url;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use trust_dns_client::rr::rdata::{SOA, TXT};
use trust_dns_client::rr::{Name, RData, Record, RecordSet, RecordType};

fn map_record(record_type: RecordType, record: String) -> Result<RData> {
    Ok(match record_type {
        RecordType::A => RData::A(record.parse()?),
        RecordType::TXT => RData::TXT(TXT::new(vec![record])),
        RecordType::NS => RData::NS(record.parse()?),
        RecordType::SOA => panic!("should never happen"), // SOA records are caught by calling function
        _ => return Err(anyhow!("RecordType::{:?} not implemented:", record_type)),
    })
}

pub async fn lookup(
    endpoint: Url,
    origin: &Name,
    name: &Name,
    record_type: RecordType,
) -> Result<RecordSet> {
    let ttl = 60;
    let serial = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u32;
    if record_type == RecordType::SOA {
        let mut record_set = RecordSet::new(origin, record_type, serial);
        record_set.insert(
            Record::from_rdata(
                origin.clone(),
                ttl,
                RData::SOA(SOA::new(
                    Name::from_str("ns")?.append_domain(origin)?,
                    Name::from_str("hostmaster")?.append_domain(origin)?,
                    serial,
                    86400,
                    7200,
                    3600000,
                    15,
                )),
            ),
            serial,
        );
        return Ok(record_set);
    }

    let request_url = build_request_url(endpoint, name, record_type)?;
    let response = reqwest::get(request_url).await?;
    let status = response.status();
    if !status.is_success() {
        return Err(anyhow!(
            "Got {}. response.text() == {:?}",
            response.status(),
            response.text().await?
        ));
    }
    let records = response.json::<Vec<String>>().await?;
    debug!("records: {:?}", records);

    let mut record_set = RecordSet::new(name, record_type, serial);
    for record in records {
        let rdata = map_record(record_type, record)?;
        record_set.insert(Record::from_rdata(name.clone(), ttl, rdata), serial);
    }
    Ok(record_set)
}

fn build_request_url(mut endpoint: Url, name: &Name, record_type: RecordType) -> Result<Url> {
    // Path must end with `/` or it will be interpreted as a filename and will be ignored during `.join()` calls
    if !endpoint.path().ends_with('/') {
        endpoint.set_path(&format!("{}/", endpoint.path()));
    }

    for label in name.iter().rev() {
        let label = std::str::from_utf8(label)?;
        endpoint = endpoint.join(&format!("{}/", label))?;
    }

    endpoint = endpoint.join(&format!("{}/", record_type))?;

    Ok(endpoint)
}

#[cfg(test)]
mod test {
    use super::*;
    use trust_dns_client::rr::dnssec::SupportedAlgorithms;

    #[test]
    fn test_build_request_url() {
        assert_eq!(
            build_request_url(
                Url::from_str("http://endpoint/").unwrap(),
                &Name::from_str("example.com").unwrap(),
                RecordType::A
            )
            .unwrap(),
            Url::from_str("http://endpoint/com/example/A/").unwrap()
        );
        assert_eq!(
            build_request_url(
                Url::from_str("http://endpoint").unwrap(),
                &Name::from_str("example.com").unwrap(),
                RecordType::A
            )
            .unwrap(),
            Url::from_str("http://endpoint/com/example/A/").unwrap()
        );

        assert_eq!(
            build_request_url(
                Url::from_str("http://endpoint/path/").unwrap(),
                &Name::from_str("example.com").unwrap(),
                RecordType::A
            )
            .unwrap(),
            Url::from_str("http://endpoint/path/com/example/A/").unwrap()
        );
        assert_eq!(
            build_request_url(
                Url::from_str("http://endpoint/path").unwrap(),
                &Name::from_str("example.com").unwrap(),
                RecordType::A
            )
            .unwrap(),
            Url::from_str("http://endpoint/path/com/example/A/").unwrap()
        );
    }

    #[test]
    fn test_map_record() {
        assert_eq!(
            map_record(RecordType::A, "192.168.0.1".to_string()).unwrap(),
            RData::A(std::net::Ipv4Addr::new(192, 168, 0, 1))
        );
        assert_eq!(
            map_record(RecordType::TXT, "abc".to_string()).unwrap(),
            RData::TXT(TXT::new(vec!["abc".to_string()]))
        );
        assert_eq!(
            map_record(RecordType::NS, "example.com".to_string()).unwrap(),
            RData::NS(Name::from_str("example.com").unwrap())
        );
    }

    #[tokio::test]
    async fn test_lookup() {
        use httpmock::prelude::*;
        let server = MockServer::start();
        let http_mock = server.mock(|when, then| {
            when.method(GET).path("/path/com/example/A/");
            then.status(200)
                .header("content-type", "text/html; charset=UTF-8")
                .body(r#"["192.168.0.1", "192.168.0.2"]"#);
        });

        let record_set = lookup(
            server.url("/path/").parse().unwrap(),
            &Name::from_str("example.com").unwrap(),
            &Name::from_str("example.com").unwrap(),
            RecordType::A,
        )
        .await
        .unwrap();

        assert_eq!(record_set.name(), &Name::from_str("example.com").unwrap());
        assert_eq!(record_set.record_type(), RecordType::A);
        assert_eq!(record_set.ttl(), 60);
        let mut records = record_set.records(true, SupportedAlgorithms::all());

        let record = records.next().unwrap();
        assert_eq!(record.name(), &Name::from_str("example.com").unwrap());
        assert_eq!(record.record_type(), RecordType::A);
        assert_eq!(record.ttl(), 60);
        assert_eq!(
            record.data(),
            Some(&RData::A(std::net::Ipv4Addr::new(192, 168, 0, 1)))
        );

        let record = records.next().unwrap();
        assert_eq!(record.name(), &Name::from_str("example.com").unwrap());
        assert_eq!(record.record_type(), RecordType::A);
        assert_eq!(record.ttl(), 60);
        assert_eq!(
            record.data(),
            Some(&RData::A(std::net::Ipv4Addr::new(192, 168, 0, 2)))
        );

        http_mock.assert();
    }
}
