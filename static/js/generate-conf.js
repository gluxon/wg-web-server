async function generateConf() {
  const { publicKey, privateKey} = window.wireguard.generateKeypair()

  const res = await window.fetch('/peers', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded'
    },
    body: `public_key=${window.encodeURIComponent(publicKey)}`
  })
  const json = await res.json()

  const file = `\
[Interface]
PrivateKey = ${privateKey}
Address = ${json.address}

[Peer]
PublicKey = ${json.server_public_key}
AllowedIPs = 0.0.0.0/0, ::0/0
Endpoint = ${window.location.hostname}:51820`

  const blob = new Blob([file])
  window.saveAs(blob, 'wg24.conf')
}
