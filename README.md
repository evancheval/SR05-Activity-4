# SR05-Activity-4

Travail de programmation demandé dans l'activité 4 de l'Unité de Valeur SR05 (Algorithmes et Systèmes répartis) à l'Université de Technologie de Compiègne.

## Description

L'application est un programme Rust qui :

- émet périodiquement un message sur `stdout` ;
- reçoit des messages depuis `stdin` ;
- écrit les logs et diagnostics sur `stderr` ;
- peut relayer le dernier message reçu au lieu d'émettre toujours le message fixe ;
- propose un mode de test pour rendre visible l'atomicité des actions.

Le message émis par défaut est `msg`.

## Comportement actuel

- **Emission périodique** : le programme écrit un message toutes les secondes sur `stdout`.
- **Sortie standard réservée au message** : seuls les messages émis passent par `stdout`.
- **Logs sur `stderr`** : les messages de réception, d'émission et de test d'atomicité sont affichés sur `stderr`.
- **Réception asynchrone** : un thread dédié bloque sur `stdin` avec `BufReader::lines()`.
- **Atomicité émission/réception** : un verrou sur `stdout` empêche que les deux actions se mélangent.
- **Couleurs** : les logs utilisent des couleurs ANSI.

## Arguments de ligne de commande

### Numéro du programme

```bash
-p <entier>
--program-number <entier>
```

Numéro utilisé pour préfixer les logs, par exemple `[1] Emission du message: ...`.

Si l'argument n'est pas fourni, la valeur par défaut est `0`.

### Test d'atomicité

```bash
-a
--test-atomicity
```

Ajoute une temporisation visible de 5 secondes pendant une émission ou une réception, avec des logs de début et de fin :

- `checking atomicity for ...`
- `finished checking atomicity for ...`

Ce mode sert à vérifier visuellement que deux actions ne s'entrecroisent pas.

### Relayer le dernier message reçu

```bash
-f
--forward-received
```

Quand cette option est activée, le programme n'émet plus systématiquement `msg` : il réémet le dernier message reçu sur `stdin`.

## Lancement

Depuis la racine du projet :

```bash
cargo run -- -p 1
```

Version silencieuse côté Cargo :

```bash
cargo run -q -- -p 1
```

## Commandes de test

### 1. Site unique

```bash
echo "Bonjour SR05" | cargo run -q -- -p 1
```

Le programme reçoit `Bonjour SR05`, l'affiche sur `stderr`, puis continue d'émettre `[1] msg` toutes les secondes sur `stdout`.

### 2. Deux programmes en pipeline

```bash
cargo run -q -- -p 1 | cargo run -q -- -p 2
```

Le premier programme émet périodiquement un message. Le second le reçoit sur son entrée standard et logue la réception.

### 3. Anneau avec FIFO

```bash
mkfifo /tmp/f ; cargo run -q -- -p 1 < /tmp/f | cargo run -q -- -p 2 | cargo run -q -- -p 3 > /tmp/f
```

Pour injecter un message dans l'anneau depuis un autre terminal :

```bash
echo "hello ring" > /tmp/f
```

### 4. Propagation du message reçu

```bash
echo "Bonjour SR05" | cargo run -q -- -p 1 -f
```

Avec `-f`, le programme réémet le dernier message reçu au lieu de continuer avec `msg`.

### 5. Test visuel de l'atomicité

```bash
echo "Bonjour SR05" | cargo run -q -- -p 1 -a
```

Le programme garde chaque action ouverte pendant environ 5 secondes pour rendre la synchronisation observable dans les logs.

## Dépendances

- `colored_text` : coloration des logs
