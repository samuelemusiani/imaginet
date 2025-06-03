# TODO

Bug in wirefilter, si blocca una volta settata la bandwith

Bug in import: non controlla che la config sia giusta e quindi se è sbagliata si spacca tutto
Bug in clear -> Dovrebbe permette di cancellare la config anche se non funziona il parsing

vde_plug ptp:///path1 ptp:///path2  se si stacca un ptp si chiude il plug. È corretto?
 
Il comando stop dovrebbe chiudere dolcemente tutto senza killare il pid :)

Stop non controlla se la roba si ferma realmente.

Completare la modalità verbosa in tutto il codice (-v -vv -vvv) per debuggare.

Fare in modo che tutto possa essere configurabile da file: inline dei comandi
di configurazione sia per switch che ns, ecc. Per vdens si può fare post-up, pre-up ad interfaccia

Check per vedere che tutti gli eseguibili siano presenti. (Check sulla versione?)

Aggiungere parametri arbitrari ai comandi per future implementazioni (senza dover
aggiornare per forza anche il codice di imaginet).

Fare test?

Più topologie?

