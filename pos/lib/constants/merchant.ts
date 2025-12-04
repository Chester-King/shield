// Merchant configuration for Shield Cafe POS
// This is a single-merchant MVP setup

export const MERCHANT_CONFIG = {
  name: "Shield Cafe",
  // Shield-managed wallet address for receiving payments (merchant@shieldcafe.com)
  address: "u1dgugpjzvd5daktef4gzll33cfh7f9vs2z473asd0qnnc93l3hj3sawrt6a50dd772fvg0af3548lsgfm455acfehel0s30kasus9mzzctmj0wm9vkz5w6k6qk9zvdk0w7jt9j0s3nhjurmjycpd0r48xy4z2tstngxrqs3tc3cghz6vd",
  // Merchant user_id for transaction polling via Shield backend
  user_id: "69a30de5-2ed2-4169-9537-aea53e7ddf70",
};

export interface POSPaymentRequest {
  v: number;           // Protocol version
  oid: string;         // Order ID (UUID)
  to: string;          // Merchant address
  amt: number;         // Amount in ZEC
  mn: string;          // Merchant name
  sum: string;         // Items summary
}
