use crate::random::{RandomNumberStream, RandomValueGenerator};
use crate::types::Decimal;

#[derive(Debug, Clone)]
pub struct Pricing {
    wholesale_cost: Decimal,
    list_price: Decimal,
    sales_price: Decimal,
    quantity: i32,
    ext_discount_amount: Decimal,
    ext_sales_price: Decimal,
    ext_wholesale_cost: Decimal,
    ext_list_price: Decimal,
    tax_percent: Decimal,
    ext_tax: Decimal,
    coupon_amount: Decimal,
    ship_cost: Decimal,
    ext_ship_cost: Decimal,
    net_paid: Decimal,
    net_paid_including_tax: Decimal,
    net_paid_including_shipping: Decimal,
    net_paid_including_shipping_and_tax: Decimal,
    net_profit: Decimal,
    refunded_cash: Decimal,
    reversed_charge: Decimal,
    store_credit: Decimal,
    fee: Decimal,
    net_loss: Decimal,
}

impl Pricing {
    pub const QUANTITY_MIN: i32 = 1;

    // Predefined markup and discount minimums
    pub fn markup_min() -> Decimal {
        Decimal::new(0, 2).unwrap()
    }

    pub fn discount_min() -> Decimal {
        Decimal::new(0, 2).unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wholesale_cost: Decimal,
        list_price: Decimal,
        sales_price: Decimal,
        quantity: i32,
        ext_discount_amount: Decimal,
        ext_sales_price: Decimal,
        ext_wholesale_cost: Decimal,
        ext_list_price: Decimal,
        tax_percent: Decimal,
        ext_tax: Decimal,
        coupon_amount: Decimal,
        ship_cost: Decimal,
        ext_ship_cost: Decimal,
        net_paid: Decimal,
        net_paid_including_tax: Decimal,
        net_paid_including_shipping: Decimal,
        net_paid_including_shipping_and_tax: Decimal,
        net_profit: Decimal,
        refunded_cash: Decimal,
        reversed_charge: Decimal,
        store_credit: Decimal,
        fee: Decimal,
        net_loss: Decimal,
    ) -> Self {
        Pricing {
            wholesale_cost,
            list_price,
            sales_price,
            quantity,
            ext_discount_amount,
            ext_sales_price,
            ext_wholesale_cost,
            ext_list_price,
            tax_percent,
            ext_tax,
            coupon_amount,
            ship_cost,
            ext_ship_cost,
            net_paid,
            net_paid_including_tax,
            net_paid_including_shipping,
            net_paid_including_shipping_and_tax,
            net_profit,
            refunded_cash,
            reversed_charge,
            store_credit,
            fee,
            net_loss,
        }
    }

    // Accessor methods
    pub fn get_wholesale_cost(&self) -> Decimal {
        self.wholesale_cost
    }

    pub fn get_list_price(&self) -> Decimal {
        self.list_price
    }

    pub fn get_sales_price(&self) -> Decimal {
        self.sales_price
    }

    pub fn get_quantity(&self) -> i32 {
        self.quantity
    }

    pub fn get_ext_discount_amount(&self) -> Decimal {
        self.ext_discount_amount
    }

    pub fn get_ext_sales_price(&self) -> Decimal {
        self.ext_sales_price
    }

    pub fn get_ext_wholesale_cost(&self) -> Decimal {
        self.ext_wholesale_cost
    }

    pub fn get_ext_list_price(&self) -> Decimal {
        self.ext_list_price
    }

    pub fn get_tax_percent(&self) -> Decimal {
        self.tax_percent
    }

    pub fn get_ext_tax(&self) -> Decimal {
        self.ext_tax
    }

    pub fn get_coupon_amount(&self) -> Decimal {
        self.coupon_amount
    }

    pub fn get_ship_cost(&self) -> Decimal {
        self.ship_cost
    }

    pub fn get_ext_ship_cost(&self) -> Decimal {
        self.ext_ship_cost
    }

    pub fn get_net_paid(&self) -> Decimal {
        self.net_paid
    }

    pub fn get_net_paid_including_tax(&self) -> Decimal {
        self.net_paid_including_tax
    }

    pub fn get_net_paid_including_shipping(&self) -> Decimal {
        self.net_paid_including_shipping
    }

    pub fn get_net_paid_including_shipping_and_tax(&self) -> Decimal {
        self.net_paid_including_shipping_and_tax
    }

    pub fn get_net_profit(&self) -> Decimal {
        self.net_profit
    }

    pub fn get_refunded_cash(&self) -> Decimal {
        self.refunded_cash
    }

    pub fn get_reversed_charge(&self) -> Decimal {
        self.reversed_charge
    }

    pub fn get_store_credit(&self) -> Decimal {
        self.store_credit
    }

    pub fn get_fee(&self) -> Decimal {
        self.fee
    }

    pub fn get_net_loss(&self) -> Decimal {
        self.net_loss
    }
}

// Limits structure for different pricing scenarios
#[derive(Debug, Clone)]
pub struct PricingLimits {
    max_quantity_sold: i32,
    max_markup: Decimal,
    max_discount: Decimal,
    max_wholesale_cost: Decimal,
}

impl PricingLimits {
    pub fn new(
        max_quantity_sold: i32,
        max_markup: Decimal,
        max_discount: Decimal,
        max_wholesale_cost: Decimal,
    ) -> Self {
        PricingLimits {
            max_quantity_sold,
            max_markup,
            max_discount,
            max_wholesale_cost,
        }
    }

    pub fn get_max_quantity_sold(&self) -> i32 {
        self.max_quantity_sold
    }

    pub fn get_max_markup(&self) -> Decimal {
        self.max_markup
    }

    pub fn get_max_discount(&self) -> Decimal {
        self.max_discount
    }

    pub fn get_max_wholesale_cost(&self) -> Decimal {
        self.max_wholesale_cost
    }
}

/// Generate pricing for sales table (store_sales, catalog_sales, web_sales)
/// This is the Rust equivalent of Java's Pricing.generatePricingForSalesTable
pub fn generate_pricing_for_sales_table(
    limits: &PricingLimits,
    stream: &mut RandomNumberStream,
) -> Pricing {
    let quantity = RandomValueGenerator::generate_uniform_random_int(
        Pricing::QUANTITY_MIN,
        limits.get_max_quantity_sold(),
        stream,
    );
    let decimal_quantity = Decimal::from_integer(quantity);
    let wholesale_cost = RandomValueGenerator::generate_uniform_random_decimal(
        Decimal::new(100, 2).unwrap(), // 1.00
        limits.get_max_wholesale_cost(),
        stream,
    );
    let ext_wholesale_cost = Decimal::multiply(decimal_quantity, wholesale_cost);

    let mut markup = RandomValueGenerator::generate_uniform_random_decimal(
        Pricing::markup_min(),
        limits.get_max_markup(),
        stream,
    );
    markup = Decimal::add2(markup, Decimal::ONE);
    let list_price = Decimal::multiply(wholesale_cost, markup);

    let mut discount = Decimal::negate(RandomValueGenerator::generate_uniform_random_decimal(
        Pricing::discount_min(),
        limits.get_max_discount(),
        stream,
    ));
    discount = Decimal::add2(discount, Decimal::ONE);
    let sales_price = Decimal::multiply(list_price, discount);
    let ext_list_price = Decimal::multiply(list_price, decimal_quantity);
    let ext_sales_price = Decimal::multiply(sales_price, decimal_quantity);
    let ext_discount_amount = Decimal::subtract(ext_list_price, ext_sales_price);

    let coupon =
        RandomValueGenerator::generate_uniform_random_decimal(Decimal::ZERO, Decimal::ONE, stream);
    let coupon_usage = RandomValueGenerator::generate_uniform_random_int(1, 100, stream);
    let coupon_amount = if coupon_usage <= 20 {
        // 20% of sales employ a coupon
        Decimal::multiply(ext_sales_price, coupon)
    } else {
        Decimal::ZERO
    };

    let net_paid = Decimal::subtract(ext_sales_price, coupon_amount);

    let shipping = RandomValueGenerator::generate_uniform_random_decimal(
        Decimal::ZERO,
        Decimal::ONE_HALF,
        stream,
    );
    let ship_cost = Decimal::multiply(list_price, shipping);
    let ext_ship_cost = Decimal::multiply(ship_cost, decimal_quantity);
    let net_paid_including_shipping = Decimal::add2(net_paid, ext_ship_cost);
    let tax_percent = RandomValueGenerator::generate_uniform_random_decimal(
        Decimal::ZERO,
        Decimal::NINE_PERCENT,
        stream,
    );
    let ext_tax = Decimal::multiply(net_paid, tax_percent);
    let net_paid_including_tax = Decimal::add2(net_paid, ext_tax);
    let net_paid_including_shipping_and_tax = Decimal::add2(net_paid_including_shipping, ext_tax);
    let net_profit = Decimal::subtract(net_paid, ext_wholesale_cost);

    // only relevant for returns
    let refunded_cash = Decimal::ZERO;
    let reversed_charge = Decimal::ZERO;
    let store_credit = Decimal::ZERO;
    let fee = Decimal::ZERO;
    let net_loss = Decimal::ZERO;

    Pricing::new(
        wholesale_cost,
        list_price,
        sales_price,
        quantity,
        ext_discount_amount,
        ext_sales_price,
        ext_wholesale_cost,
        ext_list_price,
        tax_percent,
        ext_tax,
        coupon_amount,
        ship_cost,
        ext_ship_cost,
        net_paid,
        net_paid_including_tax,
        net_paid_including_shipping,
        net_paid_including_shipping_and_tax,
        net_profit,
        refunded_cash,
        reversed_charge,
        store_credit,
        fee,
        net_loss,
    )
}

/// Predefined pricing limits for store_sales
pub fn get_store_sales_pricing_limits() -> PricingLimits {
    PricingLimits::new(100, Decimal::ONE, Decimal::ONE, Decimal::ONE_HUNDRED)
}

/// Predefined pricing limits for web_sales
pub fn get_web_sales_pricing_limits() -> PricingLimits {
    PricingLimits::new(
        100,
        Decimal::new(200, 2).unwrap(), // 2.00
        Decimal::ONE,
        Decimal::ONE_HUNDRED,
    )
}

/// Predefined pricing limits for catalog_sales
pub fn get_catalog_sales_pricing_limits() -> PricingLimits {
    // CS_QUANTITY_MAX = 100, CS_MARKUP_MAX = 2.00, CS_DISCOUNT_MAX = 1.00, CS_WHOLESALE_MAX = 100.00
    PricingLimits::new(
        100,
        Decimal::new(200, 2).unwrap(), // 2.00
        Decimal::ONE,
        Decimal::ONE_HUNDRED,
    )
}

/// Generate pricing for returns table (store_returns, catalog_returns, web_returns)
/// This is the Rust equivalent of Java's Pricing.generatePricingForReturnsTable
pub fn generate_pricing_for_returns_table(
    stream: &mut RandomNumberStream,
    quantity: i32,
    base_pricing: &Pricing,
) -> Pricing {
    let wholesale_cost = base_pricing.get_wholesale_cost();
    let list_price = base_pricing.get_list_price();
    let sales_price = base_pricing.get_sales_price();
    let tax_percent = base_pricing.get_tax_percent();
    let ext_discount_amount = base_pricing.get_ext_discount_amount();
    let coupon_amount = base_pricing.get_coupon_amount();

    let decimal_quantity = Decimal::from_integer(quantity);
    let ext_wholesale_cost = Decimal::multiply(decimal_quantity, wholesale_cost);
    let ext_list_price = Decimal::multiply(list_price, decimal_quantity);
    let ext_sales_price = Decimal::multiply(sales_price, decimal_quantity);
    let net_paid = ext_sales_price;

    let shipping = RandomValueGenerator::generate_uniform_random_decimal(
        Decimal::ZERO,
        Decimal::ONE_HALF,
        stream,
    );
    let ship_cost = Decimal::multiply(list_price, shipping);
    let ext_ship_cost = Decimal::multiply(ship_cost, decimal_quantity);
    let net_paid_including_shipping = Decimal::add2(net_paid, ext_ship_cost);
    let ext_tax = Decimal::multiply(net_paid, tax_percent);
    let net_paid_including_tax = Decimal::add2(net_paid, ext_tax);
    let net_paid_including_shipping_and_tax = Decimal::add2(net_paid_including_shipping, ext_tax);
    let net_profit = Decimal::subtract(net_paid, ext_wholesale_cost);

    // See to it that the returned amounts add up to the total returned
    // Allocate some of return to cash
    let cash_percentage = Decimal::from_integer(RandomValueGenerator::generate_uniform_random_int(
        0, 100, stream,
    ));
    let refunded_cash = Decimal::multiply(
        Decimal::divide(cash_percentage, Decimal::ONE_HUNDRED),
        net_paid,
    );

    // Allocate some to reversed charges
    let credit_percent = Decimal::from_integer(RandomValueGenerator::generate_uniform_random_int(
        1, 100, stream,
    ));
    let credit_percent = Decimal::divide(credit_percent, Decimal::ONE_HUNDRED);
    let paid_minus_refunded = Decimal::subtract(net_paid, refunded_cash);
    let reversed_charge = Decimal::multiply(credit_percent, paid_minus_refunded);

    // The rest is store credit
    let store_credit = Decimal::subtract(net_paid, reversed_charge);
    let store_credit = Decimal::subtract(store_credit, refunded_cash);

    // Pick a fee for the return
    let fee = RandomValueGenerator::generate_uniform_random_decimal(
        Decimal::ONE_HALF,
        Decimal::ONE_HUNDRED,
        stream,
    );

    // And calculate the net effect
    let net_loss = Decimal::subtract(net_paid_including_shipping_and_tax, store_credit);
    let net_loss = Decimal::subtract(net_loss, refunded_cash);
    let net_loss = Decimal::subtract(net_loss, reversed_charge);
    let net_loss = Decimal::add2(net_loss, fee);

    Pricing::new(
        wholesale_cost,
        list_price,
        sales_price,
        quantity,
        ext_discount_amount,
        ext_sales_price,
        ext_wholesale_cost,
        ext_list_price,
        tax_percent,
        ext_tax,
        coupon_amount,
        ship_cost,
        ext_ship_cost,
        net_paid,
        net_paid_including_tax,
        net_paid_including_shipping,
        net_paid_including_shipping_and_tax,
        net_profit,
        refunded_cash,
        reversed_charge,
        store_credit,
        fee,
        net_loss,
    )
}

/// Advances `stream` past the values [`generate_pricing_for_sales_table`]
/// would draw, without computing a [`Pricing`].
///
/// Must be kept in sync with [`generate_pricing_for_sales_table`]
pub fn skip_pricing_for_sales_table(stream: &mut RandomNumberStream) {
    for _ in 0..8 {
        stream.next_random();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_creation() {
        let pricing = Pricing::new(
            Decimal::new(1000, 2).unwrap(), // wholesale_cost: 10.00
            Decimal::new(1500, 2).unwrap(), // list_price: 15.00
            Decimal::new(1200, 2).unwrap(), // sales_price: 12.00
            5,                              // quantity
            Decimal::new(300, 2).unwrap(),  // ext_discount_amount: 3.00
            Decimal::new(6000, 2).unwrap(), // ext_sales_price: 60.00
            Decimal::new(5000, 2).unwrap(), // ext_wholesale_cost: 50.00
            Decimal::new(7500, 2).unwrap(), // ext_list_price: 75.00
            Decimal::new(8, 2).unwrap(),    // tax_percent: 0.08
            Decimal::new(480, 2).unwrap(),  // ext_tax: 4.80
            Decimal::new(100, 2).unwrap(),  // coupon_amount: 1.00
            Decimal::new(200, 2).unwrap(),  // ship_cost: 2.00
            Decimal::new(1000, 2).unwrap(), // ext_ship_cost: 10.00
            Decimal::new(5900, 2).unwrap(), // net_paid: 59.00
            Decimal::new(6380, 2).unwrap(), // net_paid_including_tax: 63.80
            Decimal::new(6900, 2).unwrap(), // net_paid_including_shipping: 69.00
            Decimal::new(7380, 2).unwrap(), // net_paid_including_shipping_and_tax: 73.80
            Decimal::new(900, 2).unwrap(),  // net_profit: 9.00
            Decimal::ZERO,                  // refunded_cash
            Decimal::ZERO,                  // reversed_charge
            Decimal::ZERO,                  // store_credit
            Decimal::ZERO,                  // fee
            Decimal::ZERO,                  // net_loss
        );

        assert_eq!(pricing.get_quantity(), 5);
        assert_eq!(pricing.get_wholesale_cost().get_number(), 1000);
        assert_eq!(pricing.get_list_price().get_number(), 1500);
    }

    #[test]
    fn test_pricing_limits() {
        let limits = PricingLimits::new(100, Decimal::ONE, Decimal::ONE, Decimal::ONE_HUNDRED);

        assert_eq!(limits.get_max_quantity_sold(), 100);
        assert_eq!(limits.get_max_markup(), Decimal::ONE);
    }

    #[test]
    fn test_constants() {
        assert_eq!(Pricing::QUANTITY_MIN, 1);
        assert_eq!(Pricing::markup_min().get_number(), 0);
        assert_eq!(Pricing::discount_min().get_number(), 0);
    }

    #[test]
    fn test_skip_pricing_for_sales_table_matches_generate() {
        use crate::random::RandomNumberStream;
        let mut generated = RandomNumberStream::new(1).unwrap();
        let mut skipped = RandomNumberStream::new(1).unwrap();
        generate_pricing_for_sales_table(&get_store_sales_pricing_limits(), &mut generated);
        skip_pricing_for_sales_table(&mut skipped);
        assert_eq!(generated.next_random(), skipped.next_random());
    }
}
