# tcp-proxy

Un proxy TCP bidirectionnel minimaliste. Il écoute sur une adresse locale et relaie l'intégralité du trafic vers une cible unique, dans les deux sens, jusqu'à fermeture de la connexion.

- Relais bidirectionnel complet, préserve le half-close (un sens peut se fermer sans couper l'autre).
- Une tâche par connexion : une connexion qui tombe ne tue jamais le proxy.
- Timeout de 5 s sur la connexion à la cible (une cible injoignable n'immobilise rien).
- Arrêt gracieux sur Ctrl-C : cesse d'accepter, laisse les connexions en cours se terminer.

## Usage

```
cargo run -- --listener <IP_MACHINE_A:PORT> --target <IP_TARGET:PORT>
```

Par exemple, le proxy écoute sur '10.0.10.12:8080'. Les clients se connectent à cette adresse, puis le proxy établit une connexion vers '10.10.10.12:9000' et relaie le trafic dans les deux sens.

```
cargo run -- --listener 10.0.10.12:8080 --target 10.10.10.12:9000
```

Le mode debug de cargo (`cargo run`, non optimisé) suffit pour ces usages. Pour voir le détail des connexions et le nombre d'octets relayés dans chaque sens, active les logs :

```
RUST_LOG=debug cargo run -- --listener 10.0.10.12:8080 --target 10.10.10.12:9000
```

Arrêt : `Ctrl-C`.

Options :

- `--listener <IP_MACHINE_A:PORT>` — adresse sur laquelle le proxy ecoute les connexions entrantes (IPv4 ou IPv6, ex. `[::1]:8080`).
- `--target <IP_TARGET:PORT>` — adresse de la cible vers laquelle relayer.

## Cas d'usage

### 1. Pivoting réseau (pentest)

Lors d'un test d'intrusion autorisé, la machine attaquante peut joindre une machine compromise **A** (le *pivot*), mais pas la machine **B** située dans un réseau interne. En revanche, **A peut communiquer avec B**. En lançant le proxy sur A, on relaie le trafic à travers A pour atteindre un service de B.

```
  ┌───────────┐    connexion    ┌──────────────┐     relais     ┌───────────┐
  │ Attaquant │ ──────────────▶ │  Machine A   │ ─────────────▶ │ Machine B │
  │nc <IP_A>  │     A:8445      │   (pivot)    │     B:445      │   :445    │
  │    PORT   │                 │  tcp-proxy   │                │           │
  └───────────┘                 └──────────────┘                └───────────┘
        │                                                             ▲
        └─────────────  ✗  accès direct impossible  ──────────────────┘
```

Sur A, on relaie un port vers le service visé de B :

```
cargo run -- --listener <IP_MACHINE_A:8445> --target <IP_DE_LA_CIBLE>:445
```

Depuis la machine attaquante, se connecter à `A:8445` revient alors à parler à `B:445`, en transitant par A.

> À n'utiliser que sur des systèmes que vous possédez ou êtes explicitement autorisé à tester !

### 2. Exposer un service local au réseau (hors pentest)

Un service de développement n'écoute que sur `127.0.0.1` et reste donc inaccessible depuis les autres machines. Pour le partager temporairement avec un collègue sur le réseau local, sans modifier sa configuration, on expose un port d'écoute sur le réseau local et on le relaie vers le service local :

```
cargo run -- --listener 0.0.0.0:8000 --target 127.0.0.1:3000
```

Le service, qui n'écoutait que sur `localhost:3000`, devient joignable depuis le réseau local sur le port 8000 de la machine.

## Limites

- Cible unique et fixe : pas de multi-cibles.
- Trafic relayé **en clair** : aucun chiffrement TLS. À réserver aux réseaux de confiance, ou à encapsuler dans un tunnel chiffré.
- Timeout de connexion à la cible fixé à 5 s (constante).