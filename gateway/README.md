# TacoDNS gateway

The TacoDNS gateway implements the conversion from the wire DNS protocol into an easy-to-use HTTP request/response pair.

The gateway leverages the [TrustDNS](https://github.com/bluejekyll/trust-dns) library.

## Usage

```bash
cargo run -- --http-endpoint <endpoint>
```

## HTTP request format

The HTTP request uses the GET method. The path format is the reversed domain name and the dots substituted for slashes. The path is then suffixed by the record type.

For example, a request for an `A` record on `example.com` would look like `GET /com/example/A/`.

## HTTP response format

The HTTP response is a JSON-formatted payload of an array of records. It supports three formats, text, simple, and full. The TacoDNS gateway will infer the format based on the response payload.

For example, consider this request: `GET /com/example/A/`

```json
["123.123.123.123"]
```
