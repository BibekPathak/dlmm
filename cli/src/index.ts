#!/usr/bin/env node
import { Command } from "commander";
import { registerCommands } from "./commands";

const program = new Command();

program
  .name("dlmm")
  .description("DLMM — Dynamic Liquidity Market Maker CLI")
  .version("0.1.0");

registerCommands(program);

program.parse(process.argv);

if (!process.argv.slice(2).length) {
  program.outputHelp();
}
