# SR05-Activity-4

Travail de programmation demandé dans l'activité 4 de l'Unité de Valeur SR05 (Algorithmes et Systèmes répartis) à l'Université de Technologie de Compiègne.

## Description

Ce programme Rust respecte les consignes de l'activité (hors interface graphique) :

- **Émission périodique** : le programme écrit un message toutes les secondes sur `stdout`. Le message est une chaîne de caractères fixée au lancement (`original message`).
- **Sortie standard exclusive** : aucun autre affichage n'est fait sur `stdout`. Tous les logs (réception, émission, atomicité) sont écrits sur `stderr`.
- **Réception asynchrone** : un thread dédié bloque sur la lecture de `stdin` et ne s'éveille que lorsqu'une ligne est disponible. Le programme ne sonde pas périodiquement son entrée.
- **Séquentiel et atomique** : émission et réception partagent le verrou de `stdout`. Une action en cours ne peut pas être interrompue par l'autre.

## Arguments obligatoires

### Identifiant du programme

```
-p <entier>
--program-number <entier>
```

Identifiant entier du programme, utilisé pour préfixer les logs sur `stderr` afin de distinguer les processus dans un pipeline.

## Arguments optionnels

### Test d'atomicité

```
-a
--test-atomicity
```

Ce flag active des logs supplémentaires sur `stderr` pour vérifier que les émissions et réceptions ne se chevauchent pas. Lorsqu'une action commence, un message "Début de l'action" est logué, et à la fin "Fin de l'action". En cas de chevauchement, les messages s'entremêleront, indiquant une violation de l'atomicité.

### Transmission des messages reçus

```
-f
--forward-received
```

Classiquement, le programme écrit toujours le même message sur `stdout`, indépendamment de ce qu'il reçoit. Avec ce flag, le programme émet sur `stdout` le message reçu au lieu du message fixe. Cela permet de faire circuler les messages à travers une chaîne de programmes connectés en pipeline.

## Commandes de test

> Attention : les commandes ci-dessous sont à exécuter dans un terminal compatible Unix (Linux, macOS).

Les commandes ci-dessous doivent être exécutées à la racine du projet, là où se trouve le fichier `Cargo.toml`.

### 1. Site unique — réception d'un message depuis le shell

```bash
echo "Bonjour SR05" | cargo run -q -- -p 1
```

Le programme reçoit `Bonjour SR05`, le logue sur `stderr` sous la forme `[1] Réception du message: ...`, puis continue d'émettre `[1] original message` toutes les secondes sur `stdout`.

### 2. Lien entre deux sites

```bash
cargo run -q -- -p 1 | cargo run -q -- -p 2
```

Le programme 1 émet périodiquement sur `stdout`. Le programme 2 reçoit chaque ligne sur `stdin` et la logue sur `stderr`.

### 3. Anneau (ring) avec FIFO

```bash
mkfifo /tmp/f ; cargo run -q -- -p 1 < /tmp/f | cargo run -q -- -p 2 | cargo run -q -- -p 3 > /tmp/f
```

Pour injecter un message dans l'anneau depuis un autre terminal :

```bash
echo "hello ring" > /tmp/f
```
