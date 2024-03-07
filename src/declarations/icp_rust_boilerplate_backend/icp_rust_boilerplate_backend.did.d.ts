import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type Error = { 'NotFound' : { 'msg' : string } };
export interface Rental {
  'id' : bigint,
  'daily_rate' : bigint,
  'motorcycle_brand' : string,
  'rental_date' : string,
  'rental_days' : bigint,
  'renter_name' : string,
}
export interface RentalInput {
  'daily_rate' : bigint,
  'motorcycle_brand' : string,
  'rental_date' : string,
  'rental_days' : bigint,
  'renter_name' : string,
}
export type Result = { 'Ok' : Rental } |
  { 'Err' : Error };
export interface _SERVICE {
  'add_rental' : ActorMethod<[RentalInput], [] | [Rental]>,
  'delete_rental' : ActorMethod<[bigint], Result>,
  'get_rental' : ActorMethod<[bigint], Result>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: ({ IDL }: { IDL: IDL }) => IDL.Type[];
