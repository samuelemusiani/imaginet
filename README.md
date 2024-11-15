# ImagiNet

# Dubbi

## Configurare vdens

Come riesco ad eseguire comandi/configurare il ns di vdens?

```bash
unshare --user -n -c
lsns --output-all -t user
sudo nsenter --preserve-credentials --net -t $PID
```

## Configurare switch/router con file

Dove trovo un esempio di configurazione?
Guardare cosa fanno tutte le flag
