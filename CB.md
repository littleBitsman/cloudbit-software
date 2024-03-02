# notes

Wi-Fi HTML page: /srv/http/set-wifi.html
path?: (ip)/set-wifi (check needed)

***recommended to use a guest network since the credentials are sent over basic HTTP POST without TLS***

location of software: /usr/local/lb
has:
- ADC (analog-to-digital/input)
- bit-util (utility scripts)
- button (handles button)
- cloud_client (main client code)
- comm-util (handles wifi commissioning)
- DAC (digital-to-analog/output)
- etc (contains conf and some other files)
- LEDColor (handles LED)
- mfg_test (testing scripts during development?)