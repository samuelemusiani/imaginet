# TODO

vde_plug ptp:///path1 ptp:///path2  se si stacca un ptp si chiude il plug. È corretto?
 
Start fa partire la roba anche se è già partita.

Start non controlla se tutti i device i esistono nella topologia.

Il comando stop dovrebbe chiudere dolcemente tutto senza killare il pid :)

Start inline per i namespaces

Stop non controlla se la roba si ferma realmente.

Completare la modalità verbosa in tutto il codice (-v -vv -vvv) per debuggare.

vdens in background? In modo da poter eseguire anche su macchine remote.

Fare in modo che tutto possa essere configurabile da file: inline dei comandi
di configurazione sia per switch che ns, ecc. Per vdens si può fare post-up, pre-up ad interfaccia

Check per vedere che tutti gli eseguibili siano presenti. (Check sulla versione?)

Aggiungere parametri arbitrari ai comandi per future implementazioni (senza dover
aggiornare per forza anche il codice di imaginet).

Fare test?

Più topologie?

