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
    let endpoint = {
        let mut endpoint = endpoint;

        for label in name.iter().rev() {
            let label = std::str::from_utf8(label)?;
            endpoint = endpoint.join(&format!("{}/", label))?;
        }

        endpoint = endpoint.join(&format!("{}/", record_type))?;

        endpoint
    };
    debug!("endpoint: {}", endpoint);

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

    let response = reqwest::get(endpoint).await?;
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
