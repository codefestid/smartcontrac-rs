export const idlFactory = ({ IDL }) => {
  const RentalInput = IDL.Record({
    'daily_rate' : IDL.Nat64,
    'motorcycle_brand' : IDL.Text,
    'rental_date' : IDL.Text,
    'rental_days' : IDL.Nat64,
    'renter_name' : IDL.Text,
  });
  const Rental = IDL.Record({
    'id' : IDL.Nat64,
    'daily_rate' : IDL.Nat64,
    'motorcycle_brand' : IDL.Text,
    'rental_date' : IDL.Text,
    'rental_days' : IDL.Nat64,
    'renter_name' : IDL.Text,
  });
  const Error = IDL.Variant({ 'NotFound' : IDL.Record({ 'msg' : IDL.Text }) });
  const Result = IDL.Variant({ 'Ok' : Rental, 'Err' : Error });
  return IDL.Service({
    'add_rental' : IDL.Func([RentalInput], [IDL.Opt(Rental)], []),
    'delete_rental' : IDL.Func([IDL.Nat64], [Result], []),
    'get_rental' : IDL.Func([IDL.Nat64], [Result], ['query']),
  });
};
export const init = ({ IDL }) => { return []; };
