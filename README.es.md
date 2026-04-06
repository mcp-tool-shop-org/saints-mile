<p align="center">
  <a href="README.ja.md">日本語</a> | <a href="README.zh.md">中文</a> | <a href="README.md">English</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/saints-mile/readme.png" width="400" alt="Saint's Mile" />
</p>

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/actions"><img src="https://github.com/mcp-tool-shop-org/saints-mile/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://mcp-tool-shop-org.github.io/saints-mile/"><img src="https://img.shields.io/badge/Landing_Page-saints--mile-d97706" alt="Landing Page"></a>
</p>

Un JRPG de temática fronteriza para aquellos adultos que amaron esos juegos en su momento.

Saint's Mile es un RPG de combate por turnos, ambientado en la Cuenca de Ceniza, un territorio fronterizo que está siendo transformado por el ferrocarril, el agua y la ley. Juegas como Galen Rook, un hombre cuya fama le precede, a lo largo de cuatro décadas de una vida marcada por un cartel de búsqueda emitido por otra persona.

Desarrollado en Rust para la terminal. Sin elementos gráficos innecesarios. Se centra completamente en mecánicas deterministas, combate en grupo y una historia que confía en su público.

## De qué se trata

- Un **JRPG al estilo de los años 90** con un grupo de 4 personajes, roles distintos, técnicas en dúo y combate por turnos.
- Un **western de frontera** donde la reputación es una red, la distancia influye en las decisiones y el camino es el laberinto.
- Un **juego para adultos** con temas de arrepentimiento, deber, compromiso, envejecimiento, lealtad y nuevos comienzos.
- Una **experiencia nativa para la terminal** que funciona en cualquier terminal gracias a [ratatui](https://ratatui.rs/).

## La Historia

El juego abarca casi cuatro décadas: desde un joven de 19 años que trabaja como ayudante del alguacil y aún cree que la ley y la verdad están relacionadas, hasta un joven pistolero que carga con el crimen de otra persona, pasando por un forajido maduro que cruza una cuenca moribunda con un grupo de especialistas dañados, hasta un hombre mayor que se ve obligado a decidir si una vida puede ser redimida por acciones, por la verdad, o no en absoluto.

El conflicto superficial es entre el ferrocarril, el agua y la tierra. El conflicto más profundo es quién tiene derecho a escribir la historia de lo que sucedió en Saint's Mile.

## Combate

La tensión de un enfrentamiento previo a cada batalla importante: las manos se tensan, se pone a prueba el nervio, se gana la iniciativa. Luego, entra en juego un sistema de combate JRPG basado en el grupo: cuatro miembros activos de un total de seis, cada uno con conjuntos de comandos únicos, líneas de habilidades que se profundizan a través de la historia y el vínculo, y técnicas en dúo que recompensan la inversión en el grupo.

La capa del western cambia las mecánicas, no solo la estética: munición en lugar de puntos de magia (PM), nervio en lugar de moral, resistencia en lugar de mejoras de defensa, y heridas que persisten entre las batallas.

## El Grupo

| Personaje | Rol | Identidad en el combate |
|-----------|------|----------------|
| **Galen Rook** | Tirador | Precisión, disparos calculados, mando en el campo. Evoluciona con la edad. |
| **Eli Winter** | Estafador | Ataques de nervio, interrupción, trucos baratos. La lealtad se desbloquea más adelante. |
| **Dr. Ada Mercer** | Médico | Curación, tratamiento de heridas, revelación de debilidades. |
| **Rosa Varela** | Trabajador de rancho | Control de multitudes con lazo, tanque de primera línea, presión posicional. |
| **Rev. Miriam Slate** | Predicador | Buffs canalizados, apoyo al nervio, gestión de multitudes. |
| **Lucien "Fuse" Marr** | Demolicionista | Daño de área retrasado, destrucción ambiental, remodelación del terreno. |

## Estado

**v1.0.1 — Candidata a lanzamiento.** Campaña completa implementada (Prólogo + 15 capítulos a lo largo de cuatro fases de vida). Motor de combate, sistema de duelo, encuentros de presión, gestión de estado, guardado/carga y capa de presentación TUI operativos. 151 pruebas pasando.

## Modelo de amenazas

Saint's Mile es un juego para un solo jugador que se ejecuta sin conexión. No:
- Se conecta a Internet.
- Recopila datos de telemetría o análisis.
- Accede a archivos fuera de su propio directorio de guardado.
- Requiere permisos que vayan más allá de la entrada/salida de la terminal.

Los archivos de guardado se almacenan en formato RON en un directorio accesible para el usuario.

## Requisitos

- Rust 1.80+ (edición 2021)
- Cualquier terminal con soporte para 256 colores.

## Licencia

MIT

---

Desarrollado por <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
