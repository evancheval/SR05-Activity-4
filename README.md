# SR05-Activity-4
Travail de programmation demandé dans l'activité 4 de l'Unité de Valeur SR05 (Algorithmes et Systèmes répartis) à l'Université de Technologie de Compiègne

## Programme Rust: echo stdin -> stdout

Ce projet contient un binaire Rust qui lit tout ce qui arrive sur l'entrée standard (`stdin`) puis réécrit exactement ce contenu de manière périodique sur la sortie standard (`stdout`).

Si une erreur se produit (par exemple aucune donnée reçue sur `stdin`), le message d'erreur est écrit sur la sortie d'erreur (`stderr`) et le programme quitte avec un code non nul.

Par défaut, le message est réécrit toutes les secondes jusqu'à l'arrêt du programme (`Ctrl+C`).

## Exécution sur PowerShell (Windows)

### 1) Compiler et lancer avec une entrée `stdin` (PowerShell)

```powershell
"Bonjour SR05" | cargo run --quiet
```

Le texte `Bonjour SR05` sera alors réécrit en boucle toutes les secondes.

### 2) Avec un fichier comme `stdin`

```powershell
Get-Content .\input.txt | cargo run --quiet
```

### 3) Cas d'erreur (pas d'entrée)

```powershell
cargo run --quiet
```

Le programme affichera alors une erreur sur `stderr`.

## Exécution sur Terminal (Linux/Mac)
TODO
