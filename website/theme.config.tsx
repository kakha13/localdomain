import React from 'react'
import { DocsThemeConfig } from 'nextra-theme-docs'

const config: DocsThemeConfig = {
  logo: (
    <span style={{ display: 'flex', alignItems: 'center', gap: '8px', fontWeight: 700 }}>
      <img src="/logo.svg" alt="LocalDomain" width="24" height="24" />
      LocalDomain
    </span>
  ),
  project: {
    link: 'https://github.com/kakha13/localdomain',
  },
  docsRepositoryBase: 'https://github.com/kakha13/localdomain',
  footer: {
    text: 'LocalDomain \u2014 Free local domain manager for developers.',
  },
  head: (
    <>
      <meta name="viewport" content="width=device-width, initial-scale=1.0" />
      <meta name="description" content="LocalDomain documentation - manage local domains with HTTPS, reverse proxy, and public tunnels." />
      <link rel="icon" href="/favicon.ico" />
    </>
  ),
  useNextSeoProps() {
    return {
      titleTemplate: '%s \u2013 LocalDomain Docs',
    }
  },
  sidebar: {
    defaultMenuCollapseLevel: 1,
    toggleButton: true,
  },
  toc: {
    backToTop: true,
  },
  navigation: {
    prev: true,
    next: true,
  },
}

export default config
