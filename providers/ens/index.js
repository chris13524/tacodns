const express = require('express');
const app = express();
const port = 3000;

const { ethers } = require("ethers");
const rpcAddress = "https://mainnet.infura.io/v3/f1793583b9264c7c82fc44892e9e1c46";
const provider = new ethers.providers.JsonRpcProvider(rpcAddress);

// listen for all domain and record pairs
app.use('/', async (req, res) => { // e.g. /com/example/A/
    const pathParts = req.path.replace(/\//g, " ").trim().split(' '); // e.g. ['com', 'example', 'A']
    console.log("pathParts: " + pathParts);
    const recordType = pathParts.pop(); // e.g. A
    console.log("pathParts: " + pathParts);

    // DNSLink only works with TXT records
    if (recordType != "TXT") return res.status(404).json({ error: "not TXT" });

    // DNSLink uses a prefixed label
    const _dnslink = pathParts.pop();
    if (_dnslink != "_dnslink") return res.status(404).json({ error: "not _dnslink" });

    const domain = [...pathParts].reverse().join('.'); // e.g. example.com
    console.log("domain: " + domain);
    const resolver = await provider.getResolver(domain);
    if (!resolver) return res.status(404).json({ error: "not found" });

    const url = await resolver.getText("url");
    if (!url) return res.status(404).json({ error: "url not found" });
    console.log(url);

    res.json(["dnslink=" + url]);
});

app.listen(port, "0.0.0.0", () => {
    console.log(`App listening at http://localhost:${port}`);
});
