# TODO

Add eseguibili nella configurazione in modo da poter usare imaginet anche se
il software non è installato. (Forse basta cambiare il $PATH?)

Bug in wirefilter, si blocca una volta settata la bandwith

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

