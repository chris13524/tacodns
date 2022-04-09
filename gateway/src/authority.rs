use anyhow::Result;
use futures_util::future;
use log::*;
use reqwest::{IntoUrl, Url};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use trust_dns_client::op::{LowerQuery, ResponseCode};
use trust_dns_client::rr::dnssec::{DnsSecResult, Signer, SupportedAlgorithms};
use trust_dns_client::rr::rdata::key::KEY;
use trust_dns_client::rr::{LowerName, Name, RecordType};
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
        let name: Name = name.clone().into();
        Box::pin(async move {
            crate::http::lookup(http_endpoint, &origin, &name, record_type)
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
