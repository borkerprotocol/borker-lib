/* tslint:disable */
export class JsWallet {
 free(): void;

 constructor(words?: string[]);

 words(): string[];

 child(i: number): Uint8Array;

 toBuffer(): Uint8Array;

 static fromBuffer(buf: Uint8Array): JsWallet;

}
