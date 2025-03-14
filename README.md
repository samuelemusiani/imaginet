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

## Other

Per simulare il cavo ho bisogno di avere il namespace connesso a 'nulla'
pensavo di usare ptp, hub o switch. Quale è il migliore?
- Sembra ptp ma al momento è rotto. Altrimenti si usa hub. Bisogna fare test sulle performance

Questo funziona:
vdens hub:///tmp/myhub
vdens hub:///tmp/myhub2

dpipe vde_plug 'vde:///tmpmyhub' = wirefilter -M /tmp/wr = 'vde:///tmp/myhub2'

Questo NON funziona:
vdens ptp:///tmp/myptp
vdens ptp:///tmp/myptp2

dpipe vde_plug 'ptp:///tmp/myptp' = wirefilter -M /tmp/wr = 'ptp:///tmp/myptp2'

Dopo il primo iperf muore tutto :)

BUG IN dpipe!

# TODO

## General

Il comando stop dovrebbe chiudere dolcemente tutto senza killare il pid :)

Start inline per i namespaces

Stop non controlla se la roba si ferma realmente.

Se si fa partire un namespace senza far partire lo switch l'errore non è molto chiaro.
Andrebbe controllato che la dipendenza sulla connessione sia attiva.

Completare la modalità verbosa in tutto il codice (-v -vv -vvv) per debuggare.

vdens in background? In modo da poter eseguire anche su macchine remote.

Fare in modo che tutto possa essere configurabile da file: inline dei comandi
di configurazione sia per switch che ns, ecc. Per vdens si può fare post-up, pre-up ad interfaccia

Check per vedere che tutti gli eseguibili siano presenti. (Check sulla versione?)

Aggiungere parametri arbitrari ai comandi per future implementazioni (senza dover
aggiornare per forza anche il codice di imaginet).

Fare test?

Più topologie?

## Cose nuove

### Punto punto
Esiste un plugin per il point-to-point. Posso collegare namespaces senza utilizzare
uno switch. Eseguo il comando su due terminali diversi:
```
vdens 'ptp:///tmp/myptp'
```
La point-to-point si può utilizzare per i namespace in modo da non dover dipendere
da uno switch. Si mettono tutte le interfacce su una punto-punto e si collega poi il
tutto con i vde_plug

### VXVDE
vxvde permette di fare una 'local area cloud'.
```
vdens 'vxvde:///234.0.0.1'
```
Direi che si può fare anche con i plug quindi l'idea della punto-punto rimane.

### TAP 
È possibile attaccaresi ad un interfaccia dell'host con tap. Richiede privilegi
quindi va gestita molto bene

### SLIRP
Slirp permette di connettersi ad internet senza fare interfacce nuove. Molto interessante.
```
vdens -R 10.0.2.3 slirp://
$# /sbin/udhcpc -i vde0
```
Sembra però richiesto che l'ip sia preso dal dhcp

### CMD
Esiste anche il cmd plugin per fare una specie di vpn. Passa magari tutto il
traffico tramite ssh
```
vdens cmd://"ssh berta vde_plug slirp://"
```
Al momento non sono riuscito a farlo funzionare

### Nesting plugins
Ci sono plugin come vlan e agno che ha senso farli nested. Devo valutare questa integrazione

### VDE2?
Ci sono alcune cose che mancano di vde2 (i router, autolink?, ecc). Non so se ha
senso aggiungerle

### vdeplug: hub, multi, bundling, switch 
Su libvdeplug_netnode.c c'è la possibilità di fare hub, switch, multi (?) e bunding (?).
Sono tutti unmanaged, ma si potrebbe integrarli

