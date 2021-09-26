## Example

```bash
docker-compose up --build -d
dig @127.0.0.1 example.com A
curl localhost:8000/set --data "1.1.1.1"
dig @127.0.0.1 example.com A
```
