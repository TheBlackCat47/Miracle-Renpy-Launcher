# Cahier des charges final — Miracle Ren'Py Launcher

## 1. Présentation

**Nom du projet :** Miracle Ren'Py Launcher
**Abréviation interne :** MRL
**Type :** Application desktop portable
**Plateforme initiale :** Windows 10 / Windows 11 x64
**Langage principal :** Rust
**Framework desktop :** Tauri
**Frontend :** Svelte + TypeScript + Vite
**Stockage cloud :** Google Drive
**Distribution :** exécutable portable, sans installeur
**Architecture :** locale, modulaire, sans backend central obligatoire

Miracle Ren'Py Launcher est une application desktop destinée aux jeux Ren'Py.

Son objectif principal est de fournir un système de **Cloud Saves** comparable dans son principe à Steam Cloud, mais fonctionnant directement avec le compte Google Drive personnel de l'utilisateur.

MRL doit également permettre de gérer une bibliothèque locale de jeux Ren'Py, de les lancer, d'identifier automatiquement leurs sauvegardes et de synchroniser celles-ci entre plusieurs ordinateurs.

---

# 2. Principes fondamentaux

MRL devra respecter les principes suivants :

* fonctionnement **Local First** ;
* aucune dépendance à un serveur MRL ;
* Google Drive comme stockage cloud personnel ;
* logique métier écrite en Rust ;
* frontend léger ;
* application portable ;
* architecture modulaire ;
* compatibilité ascendante ;
* aucune perte silencieuse de sauvegardes ;
* une erreur Cloud ne doit jamais empêcher de lancer un jeu ;
* aucune synchronisation de jeu lancé en dehors de MRL ;
* conservation maximale des données lorsqu'une situation est ambiguë.

---

# 3. Expérience utilisateur cible

Le fonctionnement normal doit rester extrêmement simple :

```text
Lancer MRL
↓
Connecter son compte Google
↓
Ajouter un jeu
↓
MRL détecte Ren'Py
↓
MRL détecte les sauvegardes
↓
Jouer
↓
Quitter le jeu
↓
Synchronisation automatique
```

Sur un second ordinateur :

```text
Lancer MRL
↓
Connecter le même compte Google
↓
Ajouter le même jeu
↓
MRL reconnaît le GameIdentity
↓
Téléchargement des sauvegardes
↓
Jouer
```

---

# 4. Stack technique

## Backend

* Rust stable ;
* Tauri ;
* Tokio ;
* SQLite ;
* Serde ;
* reqwest ;
* BLAKE3 ;
* tracing.

## Frontend

* Svelte ;
* TypeScript ;
* Vite ;
* CSS moderne.

Le frontend doit rester exclusivement une couche de présentation.

Il ne doit pas gérer directement :

* Google Drive ;
* OAuth ;
* SQLite ;
* fichiers locaux ;
* processus ;
* sauvegardes ;
* synchronisation ;
* credentials ;
* détection Ren'Py.

---

# 5. Architecture modulaire

Le projet devra être organisé de manière à pouvoir évoluer sans créer de dépendances fortes entre les fonctionnalités.

Structure logique recommandée :

```text
MRL
├── mrl-core
│   ├── domain
│   ├── events
│   └── common
│
├── mrl-renpy
│   ├── detection
│   ├── identity
│   └── saves
│
├── mrl-sync
│   ├── engine
│   ├── queue
│   ├── manifest
│   ├── conflicts
│   └── hashing
│
├── mrl-google-drive
│   ├── oauth
│   ├── api
│   └── storage
│
├── mrl-database
│   ├── sqlite
│   └── migrations
│
├── mrl-windows
│   ├── filesystem
│   ├── credentials
│   ├── process
│   ├── tray
│   └── startup
│
├── mrl-launcher
│   ├── games
│   └── process-monitoring
│
└── mrl-ui-tauri
    ├── commands
    ├── events
    └── frontend
```

---

# 6. Règles d'architecture

Chaque module devra exposer une API claire.

Les modules devront communiquer autant que possible via des abstractions Rust.

Exemples :

```rust
trait CloudProvider
trait CredentialStore
trait GameDetector
trait GameRepository
trait SaveRepository
trait SyncEngine
trait ProcessManager
```

Les dépendances entre modules devront rester orientées vers les abstractions.

---

# 7. Indépendance vis-à-vis de Tauri

Aucun module métier ne devra dépendre directement de Tauri.

Tauri doit uniquement servir de couche d'adaptation entre :

```text
Frontend
↕
Tauri Commands / Events
↕
Backend Rust MRL
```

Ainsi, il devra être possible ultérieurement d'ajouter :

* une CLI ;
* une autre interface ;
* des tests automatisés ;

sans réécrire le moteur métier.

---

# 8. Application portable

MRL sera distribué comme application portable.

Format principal :

```text
MiracleRenPyLauncher.exe
```

Aucun installeur ne sera requis.

L'utilisateur pourra placer l'exécutable où il le souhaite.

---

# 9. Stockage des données

Le fait que l'application soit portable ne signifie pas que toutes les données doivent nécessairement être placées à côté de l'exécutable.

Les données utilisateur devront utiliser les emplacements Windows adaptés.

Exemple :

```text
%LOCALAPPDATA%\MiracleRenPyLauncher\
```

pour :

* SQLite ;
* cache ;
* logs ;
* backups ;
* configuration locale.

Les credentials devront rester dans le stockage sécurisé Windows.

---

# 10. Gestion des chemins

MRL devra supporter :

* chemins absolus ;
* chemins relatifs.

Les chemins relatifs devront être privilégiés lorsque :

* le jeu est situé dans un dossier proche de MRL ;
* le déplacement de l'ensemble doit rester fonctionnel.

Sinon, le chemin absolu sera utilisé.

La base devra stocker suffisamment d'informations pour permettre la résolution automatique.

---

# 11. Multi-instance

Une seule instance de MRL pourra fonctionner simultanément.

Si l'utilisateur lance une seconde fois :

```text
MiracleRenPyLauncher.exe
```

MRL devra :

* détecter l'instance existante ;
* restaurer ou afficher sa fenêtre ;
* terminer la nouvelle instance.

---

# 12. Systray

MRL devra disposer d'une icône dans le systray Windows.

Menu :

```text
Ouvrir Miracle Ren'Py Launcher
Synchroniser maintenant
Mettre la synchronisation en pause
──────────────
Quitter Miracle Ren'Py Launcher
```

---

# 13. Fermeture de la fenêtre

Cliquer sur :

```text
X
```

ne devra pas arrêter MRL.

Par défaut :

```text
X → réduction dans le systray
```

La fermeture réelle devra être effectuée depuis :

```text
Quitter Miracle Ren'Py Launcher
```

Une option pourra permettre de modifier ce comportement ultérieurement.

---

# 14. Direction artistique

Le thème principal sera sombre.

Dominantes :

* bleu nuit ;
* violet ;
* noir bleuté ;
* gris sombre.

L'identité visuelle devra être légèrement cartoon, mais très modérée.

Le résultat doit rester :

* moderne ;
* gaming ;
* propre ;
* lisible ;
* légèrement ludique ;
* non enfantin.

---

# 15. Palette indicative

Exemple :

```text
Background principal     #0B1020
Background secondaire    #11172A
Cartes                   #171E35

Violet principal         #7C5CFC
Violet clair             #9D82FF

Bleu                      #4169E1
Bleu clair                #5C8DFF

Texte principal           #F5F7FF
Texte secondaire          #AAB2CE
```

La palette pourra être ajustée pendant le développement.

---

# 16. Style UI

Utiliser :

* coins arrondis ;
* cartes ;
* ombres légères ;
* contours subtils ;
* animations courtes ;
* gradients violet / bleu avec parcimonie ;
* icônes cohérentes.

Pas d'emoji utilisés comme icônes principales de l'interface.

---

# 17. Navigation

Navigation latérale recommandée :

```text
Miracle

Accueil
Bibliothèque
Synchronisation
Sauvegardes
Paramètres
```

En bas :

```text
Compte Google
État Cloud
Version MRL
```

---

# 18. Bibliothèque

La bibliothèque devra afficher les jeux sous forme de cartes.

Informations principales :

* jaquette ;
* nom ;
* état Cloud ;
* dernière utilisation ;
* bouton Jouer.

Vue grille par défaut.

Une vue liste pourra être ajoutée ultérieurement.

---

# 19. Ajout d'un jeu

L'utilisateur pourra :

* sélectionner le dossier du jeu ;
* sélectionner directement son `.exe`.

MRL devra analyser automatiquement son contenu.

---

# 20. Détection Ren'Py

MRL devra rechercher notamment :

```text
game/
renpy/
lib/
*.exe
```

ainsi que différents marqueurs caractéristiques de Ren'Py.

Le moteur devra déterminer autant que possible :

* si le jeu utilise Ren'Py ;
* son exécutable ;
* son nom ;
* sa version éventuelle ;
* les informations nécessaires à son identification.

---

# 21. GameIdentity

L'identification d'un jeu devra être indépendante autant que possible :

* du chemin d'installation ;
* du nom de l'exécutable ;
* d'une réinstallation ;
* d'une nouvelle version du jeu.

Un composant dédié devra être développé :

```text
GameIdentity
```

Il pourra exploiter :

* nom normalisé ;
* identifiant Ren'Py ;
* save directory ;
* données de configuration ;
* fichiers caractéristiques ;
* fingerprint interne.

---

# 22. Identifiant interne

Chaque jeu disposera également d'un UUID interne.

Exemple :

```text
game_uuid
```

Le système devra distinguer :

```text
GameIdentity
```

qui représente l'identité logique du jeu,

et :

```text
game_uuid
```

qui représente son enregistrement local ou Cloud.

---

# 23. Nouvelle version d'un jeu

Lorsqu'une nouvelle version d'un jeu est ajoutée, MRL devra essayer de reconnaître qu'il s'agit du même jeu.

Le changement de :

* dossier ;
* chemin ;
* executable ;
* version ;

ne devra pas automatiquement créer un nouveau jeu.

---

# 24. Détection des sauvegardes

MRL devra prendre en charge principalement :

```text
%APPDATA%\RenPy\
```

Exemple :

```text
C:\Users\User\AppData\Roaming\RenPy\MyGame-123456\
```

Les sauvegardes situées directement dans le jeu devront également pouvoir être détectées.

---

# 25. Association jeu / sauvegardes

MRL devra effectuer une détection automatique à partir de :

* nom ;
* save directory ;
* configuration Ren'Py ;
* métadonnées ;
* timestamps ;
* GameIdentity.

---

# 26. Score de confiance

Chaque association automatique devra pouvoir recevoir un score de confiance.

Exemple conceptuel :

```text
95 % → association automatique
70 % → confirmation utilisateur
40 % → association manuelle demandée
```

Les seuils seront ajustés pendant le développement.

---

# 27. Protection contre les mauvaises associations

En cas d'incertitude importante, MRL ne devra jamais synchroniser automatiquement.

Il devra demander à l'utilisateur de sélectionner le bon dossier.

Une association manuelle devra toujours être prioritaire.

---

# 28. Synchronisation du dossier complet

MRL devra synchroniser l'intégralité du dossier de sauvegarde associé au jeu.

Il ne devra pas se limiter aux fichiers :

```text
*.save
```

Exemple :

```text
MyGame-123456/
├── 1-1-LT1.save
├── 2-1-LT1.save
├── quick-1-LT1.save
├── auto-1-LT1.save
├── persistent
└── autres fichiers du jeu
```

Le fichier `persistent` devra notamment être conservé.

---

# 29. Exclusions

Le moteur de synchronisation pourra définir une liste d'exclusions.

Exemples possibles :

* fichiers temporaires ;
* lock files ;
* cache spécifique ;
* fichiers explicitement identifiés comme inutiles.

La politique par défaut devra rester conservatrice.

---

# 30. Lancement des jeux

Le bouton :

```text
Jouer
```

devra déclencher :

```text
Synchronisation entrante
↓
Lancement du jeu
↓
Surveillance du processus
↓
Jeu en cours
↓
Fermeture du jeu
↓
Analyse des sauvegardes
↓
Synchronisation sortante
```

---

# 31. Jeux lancés hors MRL

MRL ne devra pas gérer les jeux lancés directement par l'utilisateur depuis :

* Explorer ;
* raccourci ;
* autre launcher.

La synchronisation liée à une session de jeu ne concernera que les processus lancés par MRL.

---

# 32. Surveillance du processus

MRL devra :

* lancer le processus ;
* récupérer son PID ;
* suivre son état ;
* détecter sa fermeture ;
* enregistrer la durée de session.

---

# 33. Temps de jeu

MRL pourra stocker :

```text
session_start
session_end
session_duration
total_playtime
last_played
```

---

# 34. Verrouillage pendant le jeu

Lorsqu'un jeu est en cours d'exécution :

MRL ne devra pas remplacer ses sauvegardes locales par une version distante.

Aucun téléchargement destructif ne devra être effectué.

---

# 35. Synchronisation pendant le jeu

Une modification locale pourra être détectée mais les opérations dangereuses devront être différées.

Un upload pourra éventuellement être effectué après stabilisation des fichiers.

Toute réconciliation complète devra être effectuée après la fermeture du jeu.

---

# 36. Réconciliation à la fermeture

À la fermeture :

```text
Scan local
↓
Comparaison avec le manifest
↓
Comparaison Cloud
↓
Détection des changements
↓
Résolution automatique ou conflit
↓
Synchronisation
```

---

# 37. Google OAuth

MRL utilisera OAuth 2.0 pour application Desktop.

Flow :

```text
MRL
↓
Navigateur système
↓
Google
↓
Authentification
↓
Consentement
↓
Callback localhost
↓
MRL
```

Le mot de passe Google ne devra jamais être manipulé.

---

# 38. Permissions Google

Le scope devra être le plus limité possible.

Scope privilégié :

```text
drive.file
```

MRL ne devra pas demander accès à l'ensemble du Google Drive sans nécessité.

---

# 39. Credentials

MRL devra gérer :

* access token ;
* refresh token ;
* expiration ;
* compte connecté.

Les tokens ne devront jamais être stockés en clair.

---

# 40. Credential Store

Sur Windows, utiliser :

```text
Windows Credential Manager
```

ou un mécanisme sécurisé équivalent utilisant DPAPI.

---

# 41. Déconnexion Google

Lorsqu'un utilisateur déconnecte son compte Google :

* les données locales restent présentes ;
* le Cloud est désactivé ;
* les tokens sont supprimés du credential store.

---

# 42. Changement de compte Google

Passer de :

```text
Google A
```

à :

```text
Google B
```

ne devra jamais fusionner automatiquement les espaces Cloud.

MRL devra traiter les comptes comme des espaces distincts.

---

# 43. Structure Google Drive

MRL créera un dossier dédié.

Exemple :

```text
Miracle RenPy Launcher/
```

Structure :

```text
Miracle RenPy Launcher/
├── manifest.json
├── games/
│   ├── <game-id>/
│   │   ├── metadata.json
│   │   └── saves/
│   └── ...
└── devices/
    ├── <device-id>.json
    └── ...
```

---

# 44. DeviceIdentity

Chaque installation MRL disposera d'un UUID persistant.

Exemple :

```text
device_id
```

Métadonnées :

```text
device_id
device_name
os
mrl_version
last_seen
last_sync
```

---

# 45. Synchronisation bidirectionnelle

MRL devra gérer :

```text
Local → Google Drive
```

et :

```text
Google Drive → Local
```

---

# 46. Synchronisation incrémentale

Seuls les fichiers modifiés devront être transférés.

Exemple :

```text
30 fichiers
1 fichier modifié

→ 1 seul upload
```

---

# 47. BLAKE3

Les fichiers seront identifiés notamment grâce à un hash :

```text
BLAKE3
```

Métadonnées :

```text
path
size
modified_at
hash
```

---

# 48. Manifest de synchronisation

Chaque jeu devra disposer d'un manifest versionné.

Exemple :

```json
{
  "schema_version": 1,
  "game_id": "uuid",
  "revision": 12,
  "files": {
    "1-1-LT1.save": {
      "hash": "...",
      "size": 48521,
      "modified_at": 1786670231
    }
  }
}
```

---

# 49. Versioning obligatoire

Les composants suivants devront être explicitement versionnés :

* schéma SQLite ;
* manifest Google Drive ;
* fichiers de configuration ;
* protocole interne de synchronisation.

---

# 50. Compatibilité ascendante

Une nouvelle version de MRL devra pouvoir ouvrir les données créées par les versions précédentes.

Les migrations devront être automatiques lorsque possible.

Exemple :

```text
Manifest v1
↓
Migration
↓
Manifest v2
```

---

# 51. Migrations SQLite

Toutes les modifications de la base devront utiliser des migrations.

Il sera interdit de modifier directement le schéma en production sans migration versionnée.

---

# 52. Moteur de synchronisation

Le moteur devra être un module indépendant.

Responsabilités :

* scan ;
* hash ;
* comparaison ;
* upload ;
* download ;
* conflits ;
* retries ;
* historique ;
* suppression ;
* restauration.

---

# 53. File de tâches

MRL devra disposer d'une queue locale.

Types :

```text
SCAN
HASH
UPLOAD
DOWNLOAD
DELETE
RESTORE
RECONCILE
```

États :

```text
PENDING
RUNNING
COMPLETED
FAILED
RETRYING
```

---

# 54. Résilience de la queue

Un crash de MRL ne devra pas perdre l'état des opérations importantes.

Les tâches critiques devront pouvoir être persistées dans SQLite.

---

# 55. Écritures atomiques

Tous les téléchargements et restaurations devront utiliser des écritures atomiques.

Exemple :

```text
slot1.save.download
↓
téléchargement
↓
vérification hash
↓
backup de l'ancien fichier
↓
rename atomique
↓
slot1.save
```

Un fichier incomplet ne devra jamais remplacer une sauvegarde valide.

---

# 56. Crash ou coupure PC

En cas de :

* crash ;
* arrêt brutal ;
* coupure électrique ;
* fermeture forcée ;

MRL devra pouvoir reprendre proprement.

Au prochain lancement :

```text
Vérification des jobs incomplets
↓
Vérification des fichiers temporaires
↓
Reprise ou nettoyage
```

---

# 57. Gestion des conflits

Un conflit existe notamment lorsque deux appareils modifient la même sauvegarde indépendamment.

Exemple :

```text
PC-A → slot1.save version A
PC-B → slot1.save version B
```

MRL ne devra jamais choisir arbitrairement une version.

---

# 58. Conservation en cas de conflit

Les deux fichiers devront être conservés.

Exemple :

```text
slot1.save
slot1.conflict-PC-B-2026-08-14.save
```

---

# 59. Interface de conflit

L'utilisateur devra pouvoir choisir :

```text
Garder la version locale
Garder la version distante
Garder les deux
```

L'action par défaut devra privilégier la conservation.

---

# 60. Backup local

Avant toute action potentiellement destructive :

* overwrite ;
* restore ;
* suppression ;
* résolution de conflit ;

MRL devra créer un backup local.

Emplacement :

```text
%LOCALAPPDATA%\MiracleRenPyLauncher\backups\
```

---

# 61. Historique

MRL devra conserver un historique limité des versions précédentes.

Valeur initiale recommandée :

```text
5 versions
```

par fichier important.

---

# 62. Suppressions

Une suppression locale ne devra pas immédiatement supprimer définitivement la copie Cloud.

Une suppression logique devra être utilisée.

Exemple :

```text
deleted_at
```

---

# 63. Rétention

Rétention initiale recommandée :

```text
30 jours
```

avant suppression définitive d'un fichier marqué comme supprimé.

---

# 64. Mode hors ligne

MRL devra fonctionner sans connexion Internet.

L'utilisateur pourra toujours :

* ouvrir la bibliothèque ;
* lancer ses jeux ;
* utiliser les sauvegardes locales.

---

# 65. File d'attente hors ligne

Les modifications seront enregistrées localement.

État :

```text
Synchronisation en attente
```

Au retour de la connexion :

```text
Reprise automatique
```

---

# 66. Google Drive — erreurs

MRL devra gérer proprement :

* rate limit ;
* quota ;
* erreur réseau ;
* token expiré ;
* token révoqué ;
* fichier inaccessible ;
* permission refusée ;
* panne Google.

---

# 67. Backoff

Les erreurs temporaires devront utiliser un système de retry.

Exemple :

```text
1 s
5 s
15 s
60 s
```

Le système devra utiliser un backoff progressif.

---

# 68. Protection de la queue

Une erreur Google ne devra jamais bloquer définitivement la queue.

Une tâche en erreur devra être :

* marquée ;
* reportée ;
* réessayée ;
* ou nécessiter une action utilisateur.

---

# 69. Priorité au jeu

Même si Google Drive est indisponible :

```text
Jouer
```

doit rester disponible.

Exemple :

```text
Impossible de synchroniser.

[ Réessayer ]
[ Jouer hors ligne ]
```

---

# 70. Base SQLite

Tables possibles :

```text
games
game_identity
save_paths
save_files
sync_state
sync_jobs
sync_history
devices
play_sessions
settings
schema_migrations
```

---

# 71. Frontend

Structure possible :

```text
src/
├── components/
├── routes/
├── stores/
├── services/
├── assets/
├── styles/
└── App.svelte
```

---

# 72. Tauri Commands

Les actions du frontend devront utiliser des commandes explicites.

Exemples :

```text
games_add
games_list
game_launch

google_connect
google_disconnect

sync_start
sync_status

save_restore
save_list_versions
```

---

# 73. Events

Le backend devra publier des événements.

Exemples :

```text
sync://started
sync://progress
sync://completed
sync://failed

game://started
game://stopped

google://connected
google://disconnected

save://conflict
```

---

# 74. Source de vérité

La source de vérité fonctionnelle doit rester :

```text
Backend Rust + SQLite
```

Le frontend ne devra pas conserver de logique métier critique.

---

# 75. Écran Accueil

Informations possibles :

* jeux récemment joués ;
* dernière synchronisation ;
* état Google Drive ;
* jeux nécessitant une action ;
* bouton rapide Jouer.

---

# 76. Fiche jeu

Afficher :

* jaquette ;
* nom ;
* statut Ren'Py ;
* chemin ;
* save path ;
* temps de jeu ;
* dernière session ;
* dernière synchronisation ;
* état Cloud.

Actions :

```text
Jouer
Synchroniser
Sauvegardes
Paramètres
Ouvrir le dossier
```

---

# 77. Jaquettes

V1 :

* PNG ;
* JPEG ;
* WebP ;
* sélection manuelle.

En absence d'image :

MRL devra générer une carte graphique par défaut dans le frontend.

---

# 78. Écran Synchronisation

Afficher :

* compte Google ;
* statut de connexion ;
* dernière synchro ;
* jobs actifs ;
* erreurs ;
* progression ;
* appareils connus.

---

# 79. Écran Sauvegardes

Permettre :

* consultation des fichiers ;
* historique ;
* restauration ;
* ouverture du dossier local ;
* visualisation des conflits.

---

# 80. Paramètres

Sections :

```text
Général
Interface
Google Drive
Synchronisation
Sauvegardes
Notifications
Avancé
À propos
```

---

# 81. Logs

Utiliser :

```text
tracing
```

Niveaux :

```text
TRACE
DEBUG
INFO
WARN
ERROR
```

---

# 82. Rotation des logs

Les logs devront être automatiquement tournants.

Exemple :

```text
logs/
├── mrl-2026-08-14.log
├── mrl-2026-08-13.log
└── ...
```

---

# 83. Données sensibles

Ne jamais logger :

* token OAuth ;
* refresh token ;
* authorization code ;
* credentials ;
* secrets.

---

# 84. Sécurité filesystem

Toujours valider et normaliser les chemins.

Protection obligatoire contre :

```text
../
```

et autres techniques de path traversal.

---

# 85. Sécurité Cloud

Les fichiers téléchargés depuis Google Drive ne devront jamais être exécutés automatiquement.

MRL considère Google Drive uniquement comme stockage de données.

---

# 86. Async

Les opérations longues devront fonctionner hors du thread UI.

Notamment :

* réseau ;
* OAuth ;
* Google Drive ;
* hashing ;
* synchronisation ;
* SQLite si nécessaire ;
* scan important.

---

# 87. Limitation de concurrence

Limiter le nombre de transferts simultanés.

Valeur initiale possible :

```text
3 uploads
3 downloads
```

La valeur devra pouvoir être facilement modifiée.

---

# 88. Performance

Objectif de démarrage sur machine moderne :

```text
< 2 secondes
```

hors opérations réseau.

---

# 89. Consommation en idle

Lorsque MRL est dans le systray et inactif :

```text
CPU ≈ 0 %
```

La consommation mémoire devra rester raisonnable.

---

# 90. Pas de scan permanent

MRL ne devra pas scanner continuellement tous les dossiers.

Puisque les jeux lancés hors MRL ne sont pas suivis, la surveillance devra principalement être active pendant les sessions lancées par MRL et lors des phases explicites de synchronisation.

---

# 91. Mise à jour via GitHub

Le projet étant hébergé sur GitHub, les mises à jour devront être distribuées via :

```text
GitHub Releases
```

---

# 92. Vérification des mises à jour

MRL devra pouvoir interroger GitHub pour vérifier la dernière version stable.

Exemple :

```text
Version installée : 1.2.1
Version disponible : 1.3.0
```

---

# 93. Mise à jour portable

Comme MRL n'utilise pas d'installeur, le mécanisme devra pouvoir :

```text
Télécharger la nouvelle version
↓
Vérifier son intégrité
↓
Préparer le remplacement
↓
Fermer MRL
↓
Remplacer l'exécutable
↓
Relancer MRL
```

---

# 94. Sécurisation des mises à jour

Au minimum :

* hash SHA-256 ou équivalent publié ;
* vérification avant remplacement.

À terme :

* signature des releases ;
* signature du binaire Windows.

---

# 95. Canal de mise à jour

La V1 utilisera :

```text
Stable
```

L'architecture devra permettre ultérieurement :

```text
Beta
Nightly
```

sans modifier le moteur principal.

---

# 96. Version de MRL

Versionnage recommandé :

```text
Semantic Versioning
```

Exemple :

```text
1.4.2
```

---

# 97. Tests unitaires

Obligatoires sur :

* GameIdentity ;
* normalisation des chemins ;
* résolution relative / absolue ;
* hashing ;
* manifests ;
* migrations ;
* conflits ;
* comparaison de fichiers ;
* queue ;
* identification Ren'Py.

---

# 98. Tests d'intégration

Scénarios minimum :

```text
Local → Drive
Drive → Local
Offline → Online
Token expiré
Token révoqué
Quota Drive
Conflit multi-PC
Restore
Suppression
Crash pendant upload
Crash pendant download
Migration SQLite
Migration manifest
```

---

# 99. Test d'écriture atomique

Scénario :

```text
Téléchargement slot1.save
↓
MRL est brutalement arrêté à 50 %
```

Résultat attendu :

```text
slot1.save original toujours valide
```

Le `.download` incomplet sera nettoyé ou repris ultérieurement.

---

# 100. Test multi-PC

## PC A

```text
Lance le jeu avec MRL
↓
Crée slot1.save
↓
Quitte
↓
Upload
```

## PC B

```text
Ouvre MRL
↓
Ajoute le même jeu
↓
GameIdentity reconnu
↓
Download
↓
Lance le jeu
↓
slot1.save disponible
```

---

# 101. MVP MRL 1.0

Le MVP devra impérativement intégrer :

## Desktop

* application portable ;
* Tauri ;
* Svelte ;
* Rust ;
* systray ;
* instance unique ;
* fermeture vers systray.

## Jeux

* bibliothèque ;
* ajout manuel ;
* détection Ren'Py ;
* GameIdentity ;
* lancement ;
* surveillance des processus ;
* temps de jeu basique.

## Sauvegardes

* détection ;
* association ;
* score de confiance ;
* synchronisation du dossier complet ;
* `persistent` inclus ;
* hash ;
* backup local ;
* restauration ;
* conflits.

## Google

* OAuth Desktop ;
* Google Drive ;
* Credential Manager ;
* upload ;
* download ;
* synchronisation bidirectionnelle.

## Synchronisation

* manifest versionné ;
* queue ;
* incrémental ;
* retries ;
* offline ;
* écritures atomiques ;
* verrouillage pendant le jeu ;
* reconciliation après fermeture.

## Données

* SQLite ;
* migrations ;
* compatibilité ascendante ;
* configuration versionnée.

## Maintenance

* logs ;
* GitHub Releases ;
* système de mise à jour portable.

---

# 102. Hors périmètre V1

Ne sont pas requis pour la V1 :

* système de plugins dynamiques ;
* import / export de configuration ;
* mode diagnostic avancé ;
* surveillance des jeux lancés hors MRL ;
* thème clair ;
* backend central ;
* compte utilisateur MRL ;
* catalogue de jeux distant.

---

# 103. Évolutions futures

L'architecture devra permettre ultérieurement :

* métadonnées automatiques ;
* jaquettes automatiques ;
* favoris ;
* collections ;
* tags ;
* statistiques avancées ;
* Discord Rich Presence ;
* intégration F95 France ;
* gestion de mises à jour de jeux ;
* gestion de plusieurs versions d'un jeu ;
* support Linux ;
* support macOS ;
* nouveaux fournisseurs Cloud.

---

# 104. Critère de qualité de la codebase

Une nouvelle fonctionnalité ne devra pas nécessiter de modifier de nombreux modules sans raison.

Chaque domaine devra rester clairement encapsulé.

Exemple :

Ajouter ultérieurement OneDrive devra principalement nécessiter :

```text
mrl-onedrive
```

implémentant :

```rust
CloudProvider
```

sans réécriture de :

```text
mrl-sync
mrl-renpy
mrl-launcher
```

---

# 105. Critère de validation final

Le projet sera considéré fonctionnel lorsque :

1. l'utilisateur peut lancer directement `MiracleRenPyLauncher.exe` sans installation ;
2. MRL peut rester dans le systray ;
3. une deuxième instance n'est pas créée ;
4. un compte Google peut être connecté ;
5. un jeu Ren'Py peut être ajouté ;
6. MRL identifie ses sauvegardes avec suffisamment de fiabilité ;
7. le jeu peut être lancé depuis MRL ;
8. ses sauvegardes sont synchronisées après fermeture ;
9. un second PC peut récupérer ces sauvegardes ;
10. aucune écriture interrompue ne détruit une sauvegarde existante ;
11. un conflit conserve les différentes versions ;
12. une panne Google n'empêche jamais de lancer le jeu ;
13. les anciennes données MRL restent utilisables après une mise à jour ;
14. MRL peut se mettre à jour via GitHub Releases sans installeur.

---

# 106. Vision finale

Miracle Ren'Py Launcher doit fournir une expérience proche de :

**Steam Cloud + launcher léger spécialisé Ren'Py**

tout en restant :

* portable ;
* autonome ;
* léger ;
* modulaire ;
* maintenable ;
* sécurisé ;
* facilement extensible.

Pour l'utilisateur, l'utilisation idéale doit se résumer à :

```text
Ouvrir MRL
↓
Ajouter un jeu
↓
Jouer
```

La gestion des sauvegardes, du Cloud, des conflits et des différentes machines doit être prise en charge automatiquement par MRL.
