#!/bin/sh

corepack use pnpm

pnpm install

pnpm exec playwright install
pnpm exec playwright install-deps
