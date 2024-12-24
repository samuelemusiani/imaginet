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
imaginet clear -> Pulire la topologia corrente. (Fa anche stop).

Il comando stop dovrebbe chiudere dolcemente tutto senza killare il pid :)

Stop non controlla se la roba si ferma realmente.

Aggiungere quello che manca: router, vdeplug vari ed eventuali

Se si fa partire un namespace senza far partire lo switch l'errore non è molto chiaro.
Andrebbe controllato che la dipendenza sulla connessione sia attiva.

Aggiungere la modalità verbosa (-v -vv -vvv) per debuggare.

vdens in background? In modo da poter eseguire anche su macchine remote.

Fare in modo che tutto possa essere configurabile da file: inline dei comandi
di configurazione sia per switch che ns, ecc. Per vdens si può fare post-up, pre-up ad interfaccia

Check per vedere che tutti gli eseguibili siano presenti. (Check sulla versione?)

Aggiungere parametri arbitrari ai comandi per future implementazioni (senza dover
aggiornare per forza anche il codice di imaginet).

Fare test?

Più topologie?
