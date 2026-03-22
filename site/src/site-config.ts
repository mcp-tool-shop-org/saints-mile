import type { SiteConfig } from '@mcptoolshop/site-theme';

export const config: SiteConfig = {
  title: "Saint's Mile",
  description: 'A frontier JRPG for the adults who loved those games first',
  logoBadge: 'SM',
  brandName: "Saint's Mile",
  repoUrl: 'https://github.com/mcp-tool-shop-org/saints-mile',
  footerText: 'MIT Licensed — built by <a href="https://mcp-tool-shop.github.io/" style="color:var(--color-muted);text-decoration:underline">MCP Tool Shop</a>',

  hero: {
    badge: 'Rust TUI',
    headline: "Saint's Mile",
    headlineAccent: 'A frontier JRPG.',
    description: 'A weathered frontier road story with sharp banter, real consequences, and just enough mystery to make the horizon feel haunted. Built for the generation that grew up on 90s classics.',
    primaryCta: { href: 'handbook/', label: 'Read the Handbook' },
    secondaryCta: { href: 'https://github.com/mcp-tool-shop-org/saints-mile', label: 'View Source' },
    previews: [
      { label: 'Install', code: 'cargo install saints-mile' },
      { label: 'Play', code: 'saints-mile' },
      { label: 'From source', code: 'git clone https://github.com/mcp-tool-shop-org/saints-mile\ncd saints-mile && cargo run' },
    ],
  },

  sections: [
    {
      kind: 'features',
      id: 'features',
      title: 'What Kind of Game',
      subtitle: 'A frontier party RPG with terminal-native style.',
      features: [
        { title: 'Party Combat', desc: '4-slot party with distinct roles, duo techniques, and standoff-driven initiative. Not a duel game — a real 90s JRPG battle system.' },
        { title: 'Made for Adults', desc: 'Themes of regret, duty, compromise, and legacy. Adults speak like adults. Choices linger. The frontier has weight.' },
        { title: 'Terminal Native', desc: 'Runs in any terminal on earth. Zero graphics debt. Full focus on deterministic mechanics and text that trusts its audience.' },
      ],
    },
    {
      kind: 'features',
      id: 'world',
      title: 'The Cinder Basin',
      subtitle: 'A frontier territory being reshaped by rail, water, and law.',
      features: [
        { title: 'Reputation as a Web', desc: 'Helping one faction costs you with another. Towns remember who you helped and who you left behind.' },
        { title: 'Trail as Dungeon', desc: 'Distance changes decisions. Water, ammo, and horse stamina are real costs. The road between towns is where pressure lives.' },
        { title: 'Four Decades', desc: 'Play from age 19 to 50+. Galen\'s combat identity, worldview, and command presence change as life changes him.' },
      ],
    },
    {
      kind: 'code-cards',
      id: 'party',
      title: 'The Party',
      cards: [
        { title: 'Galen Rook — Gunhand', code: 'Precision • Called shots • Field command\nEvolves across four life phases:\n  Youth → speed and pride\n  Young man → harder choices\n  Adult → setup and command\n  Older → judgment and certainty' },
        { title: 'Eli Winter — Grifter', code: 'Nerve attacks • Disruption • Cheap tricks\n"Me? I\'m being charming.\n Nobody should trust that."\n\nLoyalty line unlocks late —\nthrough deed, never speeches.' },
        { title: 'Dr. Ada Mercer — Sawbones', code: 'Healing • Wound management • Diagnosis\n"He\'s one of the men bleeding\n in front of me."\n\nThe wound system matters\nbecause she exists.' },
        { title: 'Rosa Varela — Ranch Hand', code: 'Lasso CC • Front-line tanking • Guard\n"Fence doesn\'t hold itself."\n\nShe knows every fence post,\nwater point, and cattle trail.' },
        { title: 'Rev. Miriam Slate — Preacher', code: 'Channeled buffs • Nerve support • Sermon\nHer abilities work in ways that\nresist clean categorization.\n\nFaith, cadence, timing —\nor something the frontier\nhas no name for.' },
        { title: 'Lucien "Fuse" Marr — Dynamiter', code: 'Delayed AOE • Terrain reshaping\n"Every road costs somebody.\n I\'m just the cost you can see."\n\nThe player hates him first.\nThat is correct.' },
      ],
    },
  ],
};
