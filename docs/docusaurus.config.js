module.exports = {
  title: 'SoundCore-Desktop Docs',
  tagline: 'Documentation for SoundCore-Desktop',
  url: 'https://pamod-madubashana.github.io',
  baseUrl: '/SoundCore-Desktop/',
  onBrokenLinks: 'throw',
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },
  favicon: 'img/favicon.svg',
  organizationName: 'pamod-madubashana',
  projectName: 'SoundCore-Desktop',
  deploymentBranch: 'gh-pages',
  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          routeBasePath: '/',
          sidebarPath: require.resolve('./sidebars.js'),
          editUrl: 'https://github.com/pamod-madubashana/SoundCore-Desktop/edit/main/docs/',
        },
        theme: {
          customCss: require.resolve('./src/css/custom.css'),
        },
      },
    ],
  ],
  themeConfig: {
    colorMode: {
      defaultMode: 'dark',
      disableSwitch: false,
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'SoundCore-Desktop Docs',
      logo: {
        alt: 'SoundCore-Desktop',
        src: 'img/favicon.svg',
      },
      items: [
        {
          type: 'doc',
          docId: 'intro',
          position: 'left',
          label: 'Docs',
        },
        {
          type: 'doc',
          docId: 'downloads',
          position: 'left',
          label: 'Downloads',
        },
        {
          href: 'https://github.com/pamod-madubashana/SoundCore-Desktop',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            { label: 'Introduction', to: '/' },
            { label: 'Installation', to: '/installation' },
            { label: 'Downloads', to: '/downloads' },
            { label: 'Deployment', to: '/deployment' },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/pamod-madubashana/SoundCore-Desktop',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} SoundCore-Desktop.`, 
    },
  },
};
