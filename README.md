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

Dove trovo un esempio di configurazione? Uguale ai comandi che dai (in teoria)

socat UNIZ:/path STDIO

Guardare cosa fanno tutte le flag

vdeterm permette la configurazione con le freccette

wirefilter -> Cavo vde
vdeplug4 ?

# TODO

rsnet ps --> Comando per controllare lo stato della rete? (Più che
    altro va risolto il problema degli switch che rimangono nel sistema
    avendoli messi come demoni). PID FILE!!

rsnet exec -> Eseguire un comanod in uno switch/router con il socket di
    management

Aggiungere quello che manda: router, cavi, vdeplug vari ed eventuali

Fare in modo che tutto possa essere configurabile da file (+ inline?)
