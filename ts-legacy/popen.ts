import { runCli } from './src/cli';

runCli(process.argv, { defaultMode: 'popn' }).catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
