# RCLI

## Homework Check

```bash
rcli genpass -l 32 > fixtures/chacha20.txt
rcli text encrypt --key fixtures/chacha20.txt # input from stdin
rcli text decrypt --key fixtures/chacha20.txt # input from stdin
```

```bash
rcli jwt sign --sub acme --aud device1 --exp 1m # exp支持 d(day)/h(hour)/m(minute)/s(second)
rcli jwt verify -t {token}
```

```bash
rcli http serve
# Visit http://0.0.0.0:8080/src
```
