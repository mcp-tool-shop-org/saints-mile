<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.md">English</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/saints-mile/readme.png" width="400" alt="Saint's Mile" />
</p>

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/actions"><img src="https://github.com/mcp-tool-shop-org/saints-mile/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://mcp-tool-shop-org.github.io/saints-mile/"><img src="https://img.shields.io/badge/Landing_Page-saints--mile-d97706" alt="Landing Page"></a>
</p>

Un JRPG ambientato in una frontiera, pensato per gli adulti che hanno amato quei giochi.

Saint's Mile è un RPG a turni con un gruppo di personaggi, ambientato nella regione della Cinder Basin, un territorio di frontiera in fase di trasformazione grazie alle ferrovie, all'acqua e alla legge. Si interpreta Galen Rook, un uomo la cui fama lo precede, attraverso quattro decenni di una vita vissuta sotto il peso di un mandato di cattura scritto da qualcun altro.

Sviluppato in Rust per l'utilizzo in terminale. Nessuna grafica pesante. Massima attenzione alla meccanica deterministica, al combattimento di gruppo e a una storia che si rivolge a un pubblico maturo.

## Di cosa si tratta

- Un **JRPG in stile anni '90** con un gruppo di 4 personaggi, ruoli distinti, tecniche a coppie e combattimenti a turni.
- Un **western ambientato in una frontiera** dove la reputazione è una rete complessa, la distanza influenza le decisioni e il percorso è una prigione.
- Un **gioco per adulti** che affronta temi come il rimpianto, il dovere, i compromessi, l'invecchiamento, la lealtà e la possibilità di ricominciare.
- Un'**esperienza nativa per terminale** che funziona in qualsiasi terminale grazie a [ratatui](https://ratatui.rs/).

## La storia

Il gioco si sviluppa nell'arco di quasi quattro decenni: dalla vita di un giovane vice di 19 anni che crede ancora che la legge e la verità siano correlate, a un giovane pistolero che porta con sé i crimini di qualcun altro, a un fuorilegge maturo che attraversa una regione in declino con un gruppo di specialisti provati, fino a un uomo più anziano costretto a decidere se una vita possa essere redenta attraverso le azioni, la verità o nulla.

Il conflitto principale è rappresentato dalle ferrovie, dall'acqua e dalla terra. Il conflitto più profondo è chi ha il diritto di scrivere la storia di ciò che è accaduto a Saint's Mile.

## Combattimento

La tensione del "braccio di ferro" precede ogni combattimento importante: le mani si stringono, i nervi sono messi alla prova, l'iniziativa viene guadagnata. Successivamente, entra in gioco un sistema di combattimento JRPG basato sul gruppo: quattro membri attivi su un totale di sei, ognuno con set di comandi unici, abilità che si approfondiscono attraverso la storia e il legame, e tecniche a coppie che premiano l'investimento nel gruppo.

L'elemento western modifica le meccaniche, non solo l'estetica: munizioni al posto dei punti magia (MP), nervi al posto del morale, resistenza al posto delle buff di difesa, ferite che persistono tra i combattimenti.

## Il gruppo

| Personaggio | Ruolo | Identità in battaglia |
|-----------|------|----------------|
| **Galen Rook** | Cecchino | Precisione, colpi mirati, comando sul campo. Si evolve con l'età. |
| **Eli Winter** | Truffatore | Attacchi nervosi, interruzioni, sotterfugi. La lealtà si sblocca più tardi. |
| **Dr. Ada Mercer** | Medico | Cura, gestione delle ferite, rivelazione delle debolezze. |
| **Rosa Varela** | Bracciante di ranch | Controllo della folla con la fune, resistenza in prima linea, pressione posizionale. |
| **Rev. Miriam Slate** | Predicatore | Buff potenziati, supporto nervoso, gestione della folla. |
| **Lucien "Fuse" Marr** | Minatore | Danni ad area ritardati, distruzione ambientale, rimodellamento del terreno. |

## Stato di avanzamento

**v1.0.1 — Release Candidate.** Campagna completa implementata (prologo + 15 capitoli attraverso quattro fasi della vita). Motore di combattimento, sistema di stallo, incontri di pressione, gestione dello stato, salvataggio/caricamento e livello di presentazione TUI tutti operativi. 151 test superati.

## Modello di minaccia

Saint's Mile è un gioco per giocatore singolo e offline. Non:
- Si connette a Internet
- Raccoglie dati di telemetria o analisi
- Accede a file al di fuori della propria directory di salvataggio
- Richiede autorizzazioni diverse dall'input/output del terminale.

I file di salvataggio sono memorizzati in formato RON in una directory accessibile all'utente.

## Requisiti

- Rust 1.80+ (edizione 2021)
- Qualsiasi terminale con supporto per colori a 256

## Licenza

MIT

---

Sviluppato da <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
