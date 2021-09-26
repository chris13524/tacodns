# DynamicDNS provider

This provider will provide DNS records to TacoDNS sourced from a dynamically-updatable database. It implements an API endpoint that authorized users can use to update their IP address.

When the IP address is not provided, it will be inferred based on the connecting IP or the `Forwarded` header.
