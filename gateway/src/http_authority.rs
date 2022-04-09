use anyhow::{anyhow, Result};
use log::*;
use reqwest::{IntoUrl, Url};

use futures_util::future;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use trust_dns_client::op::{LowerQuery, ResponseCode};
use trust_dns_client::rr::dnssec::{DnsSecResult, Signer, SupportedAlgorithms};
use trust_dns_client::rr::rdata::{key::KEY, SOA, TXT};
use trust_dns_client::rr::{LowerName, Name, RData, Record, RecordSet, RecordType};
use trust_dns_server::authority::{
    AuthLookup, Authority, LookupError, LookupRecords, MessageRequest, UpdateResult, ZoneType,
};

pub struct HttpAuthority {
    origin: LowerName,
    http_endpoint: Url,
}

impl HttpAuthority {
    pub fn new<U: IntoUrl>(origin: String, http_endpoint: U) -> Result<HttpAuthority> {
        Ok(HttpAuthority {
            origin: LowerName::from(Name::from_ascii(origin)?),
            http_endpoint: http_endpoint.into_url()?,
        })
    }
}

impl Authority for HttpAuthority {
    type Lookup = AuthLookup;
    type LookupFuture = future::Ready<Result<Self::Lookup, LookupError>>;

    fn zone_type(&self) -> ZoneType {
        ZoneType::Primary
    }

    fn is_axfr_allowed(&self) -> bool {
        false
    }

    fn update(&mut self, _update: &MessageRequest) -> UpdateResult<bool> {
        Err(ResponseCode::NotImp)
    }

    fn origin(&self) -> &LowerName {
        &self.origin
    }

    fn lookup(
        &self,
        name: &LowerName,
        record_type: RecordType,
        is_secure: bool,
        supported_algorithms: SupportedAlgorithms,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Lookup, LookupError>> + Send>> {
        let http_endpoint = self.http_endpoint.clone();
        let origin: Name = self.origin().into();
        let name = name.clone();
        Box::pin(async move {
            lookup_impl(http_endpoint, &origin, &name, record_type)
                .await
                .map_err(|e| {
                    error!("Error in lookup_impl: {}", e);
                    LookupError::NameExists
                })
                .map(|record_set| {
                    AuthLookup::answers(
                        LookupRecords::new(is_secure, supported_algorithms, Arc::new(record_set)),
                        None,
                    )
                })
        })
    }

    fn search(
        &self,
        query: &LowerQuery,
        is_secure: bool,
        supported_algorithms: SupportedAlgorithms,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Lookup, LookupError>> + Send>> {
        Box::pin(self.lookup(
            query.name(),
            query.query_type(),
            is_secure,
            supported_algorithms,
        ))
    }

    fn get_nsec_records(
        &self,
        _name: &LowerName,
        _is_secure: bool,
        _supported_algorithms: SupportedAlgorithms,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Lookup, LookupError>> + Send>> {
        Box::pin(future::ok(AuthLookup::default()))
    }

    fn add_update_auth_key(&mut self, _name: Name, _key: KEY) -> DnsSecResult<()> {
        Err("DNSSEC not implemented.".into())
    }

    fn add_zone_signing_key(&mut self, _signer: Signer) -> DnsSecResult<()> {
        Err("DNSSEC not implemented.".into())
    }

    fn secure_zone(&mut self) -> DnsSecResult<()> {
        Err("DNSSEC not implemented.".into())
    }
}

async fn lookup_impl(
    http_endpoint: Url,
    origin: &Name,
    name: &LowerName,
    record_type: RecordType,
) -> Result<RecordSet> {
    let name: Name = name.into();

    let endpoint = {
        let mut endpoint = http_endpoint;

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
        let rdata = match record_type {
            RecordType::A => RData::A(record.parse()?),
            RecordType::TXT => RData::TXT(TXT::new(vec![record])),
            RecordType::NS => RData::NS(record.parse()?),
            RecordType::SOA => panic!("should never happen"),
            _ => return Err(anyhow!("RecordType::{:?} not implemented:", record_type)),
        };
        record_set.insert(Record::from_rdata(name.clone(), ttl, rdata), serial);
    }
    Ok(record_set)
}
