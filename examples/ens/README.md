## Example

```bash
docker-compose up --build -d
dig @127.0.0.1 _dnslink.vitalik.eth TXT
dig _dnslink.vitalik.eth.link TXT
nslookup -type=TXT _dnslink.vitalik.eth 127.0.0.1
nslookup -type=TXT _dnslink.vitalik.eth.link

docker-compose down && docker-compose up --build -d && sleep 1 && nslookup -type=TXT _dnslink.vitalik.eth 127.0.0.1 && docker-compose logs -f
```
