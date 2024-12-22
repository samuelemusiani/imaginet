# ImagiNet

Main:
    - Config: Parsing del yaml. Controlli vari ed eventuali + errori
    - VDE: Modulo responsabile di generare effettivamente la rete.
        Lo divido in moduli per ogni componente di vde e faccio in modo
        che sia generale la possibilità di generare la rete

# Dubbi

## Configurare vdens

Come riesco ad eseguire comandi/configurare il ns di vdens?

```bash
unshare --user -n -c
lsns --output-all -t user
sudo nsenter --preserve-credentials --net -t $PID
```

Si può entrare...
```bash
vdens /tmp/sw1
echo $$ # To get id
nsenter -t $PID --preserve-credentials -U -n --keep-caps
```

## Configurare switch/router con file

Dove trovo un esempio di configurazione? Uguale ai comandi che dai (in teoria)

socat UNIZ:/path STDIO

Guardare cosa fanno tutte le flag

vdeterm permette la configurazione con le freccette

wirefilter -> Cavo vde
vdeplug4 ?

# TODO

imaginet exec -> Eseguire un comanod in uno switch/router con il socket di management.
imaginet add -> Aggiungere un componente alla rete corrente.
imaginet rm -> Aggiungere un componente alla rete corrente.
imaginet dump -> Dump della rete corrente in file di configurazione.
imaginet stop _ -> Stoppare un determinato componente della rete.
imaginet start _ -> Far partire un determinato componente della rete se fermato.
imaginet clear -> Pulire la topologia corrente. (Fa anche stop).

Il comando stop dovrebbe chiudere dolcemente tutto senza killare il pid :)

Wirefilter configurato con un file.

Stop non controlla se la roba si ferma realmente.

Aggiungere quello che manda: router, vdeplug vari ed eventuali

Aggiungere la modalità verbosa (-v -vv -vvv) per debuggare.

Iniline execution per attach (fare in modo che il comando non apra un nuovo terminale)

vdens in background? In modo da poter eseguire anche su macchine remote.

Aggiungere la possibilità di avere switch hub (non devono aprire il manegement?)

Fare in modo che tutto possa essere configurabile da file: inline dei comandi
di configurazione sia per switch che ns, ecc. Per vdens si può fare post-up, pre-up ad interfaccia

Check per vedere che tutti gli eseguibili siano presenti. (Check sulla versione?)

Aggiungere parametri arbitrari ai comandi per future implementazioni (senza dover
aggiornare per forza anche il codice di imaginet).

Generalizzare il terminale (no foot) :)

Fare test?

Più topologie?
