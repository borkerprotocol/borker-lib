/* tslint:disable */
export class JsWallet {

  free(): void;

  constructor(words?: string[]);

  words(): string[];

  childAt(derivationPath: number[]): JsChildWallet;

  toBuffer(): Uint8Array;

  static fromBuffer(buf: Uint8Array): JsWallet;

}

export enum Network {
  Dogecoin,
  Litecoin,
  Bitcoin,
}

export class JsChildWallet {

  free(): void;

  address(network: Network): string;

}