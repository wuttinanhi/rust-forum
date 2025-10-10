#!/bin/sh

corepack use pnpm

pnpm install

pnpm exec playwright install --with-deps
pnpm exec playwright test --reporter null
