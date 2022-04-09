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
                    Name::from_str("ns")?.append_domain(origin),
                    Name::from_str("hostmaster")?.append_domain(origin),
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

    let mut record_set = RecordSet::new(&name, record_type, serial);
    for record in records {
        let rdata = map_record(record_type, record)?;
        record_set.insert(Record::from_rdata(name.clone(), ttl, rdata), serial);
    }
    Ok(record_set)
}

fn build_request_url(mut endpoint: Url, name: &Name, record_type: RecordType) -> Result<Url> {
    // Path must end with `/` or it will be interpreted as a filename and will be ignored during `.join()` calls
    if !endpoint.path().ends_with("/") {
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
}
