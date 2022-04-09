use anyhow::Result;
use log::*;
use reqwest::{IntoUrl, Url};
use std::sync::Arc;
use trust_dns_client::op::ResponseCode;
use trust_dns_client::rr::{LowerName, Name, RecordType};
use trust_dns_server::authority::LookupOptions;
use trust_dns_server::authority::{
    AuthLookup, Authority, LookupError, LookupRecords, MessageRequest, UpdateResult, ZoneType,
};
use trust_dns_server::server::RequestInfo;

pub struct HttpAuthority {
    origin: LowerName,
    endpoint: Url,
}

impl HttpAuthority {
    pub fn new<U: IntoUrl>(origin: String, endpoint: U) -> Result<HttpAuthority> {
        Ok(HttpAuthority {
            origin: LowerName::from(Name::from_ascii(origin)?),
            endpoint: endpoint.into_url()?,
        })
    }
}

#[async_trait::async_trait]
impl Authority for HttpAuthority {
    type Lookup = AuthLookup;

    fn zone_type(&self) -> ZoneType {
        ZoneType::Primary
    }

    fn is_axfr_allowed(&self) -> bool {
        false
    }

    async fn update(&self, _update: &MessageRequest) -> UpdateResult<bool> {
        Err(ResponseCode::NotImp)
    }

    fn origin(&self) -> &LowerName {
        &self.origin
    }

    async fn lookup(
        &self,
        name: &LowerName,
        query_type: RecordType,
        lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        let endpoint = self.endpoint.clone();
        let origin: Name = self.origin().into();
        let name: Name = name.clone().into();
        crate::http::lookup(endpoint, &origin, &name, query_type)
            .await
            .map_err(|e| {
                error!("Error in lookup_impl: {}", e);
                LookupError::NameExists
            })
            .map(|record_set| {
                AuthLookup::answers(
                    LookupRecords::new(lookup_options, Arc::new(record_set)),
                    None,
                )
            })
    }

    async fn search(
        &self,
        request_info: RequestInfo<'_>,
        lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        self.lookup(
            request_info.query.name(),
            request_info.query.query_type(),
            lookup_options,
        )
        .await
    }

    async fn get_nsec_records(
        &self,
        _name: &LowerName,
        _lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        Ok(AuthLookup::default())
    }

    // fn add_update_auth_key(&mut self, _name: Name, _key: KEY) -> DnsSecResult<()> {
    //     Err("DNSSEC not implemented.".into())
    // }

    // fn add_zone_signing_key(&mut self, _signer: Signer) -> DnsSecResult<()> {
    //     Err("DNSSEC not implemented.".into())
    // }

    // fn secure_zone(&mut self) -> DnsSecResult<()> {
    //     Err("DNSSEC not implemented.".into())
    // }
}
