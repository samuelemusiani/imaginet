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

