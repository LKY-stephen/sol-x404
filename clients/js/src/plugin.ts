import { UmiPlugin } from '@metaplex-foundation/umi';
import { createSolX404Program } from './generated';

export const solX404 = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createSolX404Program(), false);
  },
});
