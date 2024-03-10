Ceci est ûn projet pedagogique qui demande la création d'un LoadBalancer en rust.

Pour cet exercice, on part sur un service web, hébergé sur deux serveurs ou plus.

Notre LoadBalancer écoute sur le port 80 et retransmets selon les serveurs disponibles.

Pour lancer le projet il vous faut rust d'installer sur votre serveur / ordinateur.

Une fois installé, il suffit de faire :
```
cargo build
cargo run
```

Et le LoadBalancer sera en marche.
