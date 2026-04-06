<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.md">English</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/saints-mile/readme.png" width="400" alt="Saint's Mile" />
</p>

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/actions"><img src="https://github.com/mcp-tool-shop-org/saints-mile/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://mcp-tool-shop-org.github.io/saints-mile/"><img src="https://img.shields.io/badge/Landing_Page-saints--mile-d97706" alt="Landing Page"></a>
</p>

Um JRPG de temática fronteiriça para os adultos que amavam esses jogos.

Saint's Mile é um RPG de combate por turnos, ambientado na Bacia de Cinzas, um território fronteiriço sendo transformado por ferrovias, rios e leis. Você joga como Galen Rook, um homem cuja reputação o precede, ao longo de quatro décadas de uma vida vivida sob a sombra de um mandado de prisão escrito por outra pessoa.

Desenvolvido em Rust para o terminal. Sem elementos gráficos desnecessários. Foco total em mecânicas determinísticas, combate em grupo e uma história que confia em seu público.

## O que é isso

- Um **JRPG no estilo dos anos 90** com um grupo de 4 personagens, papéis distintos, técnicas em dupla e combate por turnos.
- Um **western de fronteira** onde a reputação é uma teia, a distância muda as decisões e o caminho é o calabouço.
- Um **jogo para adultos** — aborda temas de arrependimento, dever, compromisso, envelhecimento, lealdade e recomeço.
- Uma **experiência nativa para o terminal** — funciona em qualquer terminal, utilizando [ratatui](https://ratatui.rs/)

## A História

O jogo abrange quase quatro décadas: desde um jovem de 19 anos, delegado que ainda acredita que a lei e a verdade estão relacionadas, até um jovem pistoleiro que carrega o crime de outra pessoa, passando por um foragido experiente que atravessa uma região em decadência com um grupo de especialistas problemáticos, e finalmente, um homem mais velho forçado a decidir se uma vida pode ser redimida por ações, pela verdade ou não.

O conflito superficial é entre ferrovias, rios e terras. O conflito mais profundo é quem tem o direito de escrever a história do que aconteceu em Saint's Mile.

## Combate

A tensão do confronto precede cada batalha importante — as mãos tremem, a coragem é testada, a iniciativa é conquistada. Em seguida, um sistema de batalha JRPG baseado em grupo entra em ação: quatro membros ativos de um grupo de seis, cada um com conjuntos de comandos únicos, habilidades que se aprofundam através da história e do vínculo, e técnicas em dupla que recompensam o investimento no grupo.

A temática do western muda as mecânicas, não apenas a aparência: munição em vez de MP, nervos em vez de moral, resistência em vez de bônus de defesa, ferimentos que persistem entre as batalhas.

## O Grupo

| Personagem | Papel | Identidade em Batalha |
|-----------|------|----------------|
| **Galen Rook** | Atirador | Precisão, tiros certeiros, comando de campo. Evolui com a idade. |
| **Eli Winter** | Trapaceiro | Ataques de nervos, interrupção, truques baratos. A lealdade é desbloqueada mais tarde. |
| **Dr. Ada Mercer** | Médico | Cura, tratamento de ferimentos, revelação de fraquezas. |
| **Rosa Varela** | Fazendeiro | Controle de multidão com laço, tanque na linha de frente, pressão posicional. |
| **Rev. Miriam Slate** | Pregador | Bônus canalizados, suporte de nervos, gerenciamento de multidões. |
| **Lucien "Fuse" Marr** | Mineiro | Dano em área atrasado, destruição ambiental, transformação do terreno. |

## Status

**v1.0.1 — Release Candidate.** Campanha completa implementada (Prólogo + 15 capítulos ao longo de quatro fases da vida). Motor de combate, sistema de duelo, encontros de pressão, gerenciamento de estado, salvar/carregar e camada de apresentação TUI operacionais. 151 testes passando.

## Modelo de Ameaça

Saint's Mile é um jogo para um único jogador, offline. Ele não:
- Conecta-se à internet
- Coleta dados de telemetria ou análises
- Acessa arquivos fora de seu próprio diretório de salvamento
- Requer qualquer permissão além da entrada/saída do terminal

Os arquivos de salvamento são armazenados em formato RON em um diretório acessível ao usuário.

## Requisitos

- Rust 1.80+ (edição 2021)
- Qualquer terminal com suporte a 256 cores

## Licença

MIT

---

Desenvolvido por <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
