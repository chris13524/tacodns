# DNS-01 ACME challenge provider

This ACME provider implements the [HTTP request](https://go-acme.github.io/lego/dns/httpreq/) DNS provider for LEGO which can be used with ingress servers such as Traefik.

This provider will provide TXT DNS records to TacoDNS under the prefix `_acme-challenge.`.
