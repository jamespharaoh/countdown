use std::cmp::Reverse;
use std::fmt;
use std::iter;
use std::rc::Rc;

fn main () {

	let mut available_large = vec! [ 25, 50, 75, 100 ];

	let mut available_small = (1 ..= 10).map (
		|val| iter::repeat (val).take (2),
	).flatten ().collect::<Vec <u64>> ();

	let num_large = rand::random::<usize> () % 5;
	let num_small = 6 - num_large;

	if num_large > 0 {
		println! ("Contestant chooses: {} large and {} small", num_large, num_small);
	} else {
		println! ("Contestant chooses: {} small", num_small);
	}

	let mut numbers: Vec <u64> = Iterator::chain (
		iter::repeat_with (|| take_random (& mut available_small)).take (num_small),
		iter::repeat_with (|| take_random (& mut available_large)).take (num_large),
	).collect ();

	numbers.sort ();

	print! ("Numbers drawn: ");
	for (index, number) in numbers.iter ().enumerate () {
		if index > 0 { print! (", "); }
		print! ("{}", number);
	}
	print! ("\n");

	let target = 100 + rand::random::<u64> () % 900;

	println! ("Random target: {}", target);

	let expressions = numbers.iter ().map (
		|& number| Expression::constant (number),
	).collect::<Vec <_>> ();

	let solutions = combine (& expressions).iter ().skip_while (
		|expr| expr.value () < target,
	).take_while (
		|expr| expr.value () == target,
	).cloned ().collect::<Vec <Expression>> ();

	let (
		simple_solutions,
		redundant_solutions,
	) = solutions.into_iter ().partition::<Vec <_>, _> (
		|solution| ! solution.redundant (),
	);

	println! ("Simple solutions found: {}", simple_solutions.len ());

	for solution in simple_solutions {
		println! (" • {}", solution);
	}

	println! ("Redundant solutions found: {}", redundant_solutions.len ());

	for solution in redundant_solutions {
		println! (" • {}", solution);
	}

}

fn take_random (available: & mut Vec <u64>) -> u64 {
	let index = rand::random::<usize> () % available.len ();
	available.remove (index)
}

fn combine (expressions: & [Expression]) -> Vec <Expression> {

	if expressions.is_empty () {
		return vec! ();
	}

	if expressions.len () == 1 {
		return vec! [ expressions [0].clone () ];
	}

	if expressions.len () == 2 {
		return combine_two (& expressions [0], & expressions [1]);
	}

	let mut result: Vec <Expression> = Vec::new ();

	for index in 0 .. expressions.len () {

		let this = & expressions [index];

		let rest = Iterator::chain (
			expressions [.. index].iter (),
			expressions [index + 1 ..].iter (),
		).cloned ().collect::<Vec <_>> ();

		for combination in combine (& rest) {
			result.extend (combine_two (this, & combination));
		}

	}

	canonicalise (& mut result);
	result

}

fn combine_two (expr1: & Expression, expr2: & Expression) -> Vec <Expression> {

	let mut result: Vec <Expression> = Vec::new ();

	let val1 = expr1.value ();
	let val2 = expr2.value ();

	result.push (expr1.clone ());
	result.push (expr2.clone ());

	result.push (Expression::add (expr1, expr2));

	if val1 > val2 {
		result.push (Expression::subtract (expr1.clone (), expr2.clone ()));
	}

	if val2 > val1 {
		result.push (Expression::subtract (expr2.clone (), expr1.clone ()));
	}

	result.push (Expression::multiply (expr1.clone (), expr2.clone ()));

	if val1 % val2 == 0 {
		result.push (Expression::divide (expr1.clone (), expr2.clone ()));
	}

	if val2 % val1 == 0 {
		result.push (Expression::divide (expr2.clone (), expr1.clone ()));
	}

	canonicalise (& mut result);
	result

}

fn canonicalise (values: & mut Vec <Expression>) {
	values.sort ();
	values.dedup ();
}

#[ derive (Clone, Eq, Ord, PartialEq, PartialOrd) ]
struct Expression {
	inner: Rc <ExpressionData>,
}

#[ derive (Eq, Ord, PartialEq, PartialOrd) ]
struct ExpressionData {
	value: u64,
	redundant: bool,
	source: ExpressionType,
}

impl Expression {

	fn constant (value: u64) -> Expression {
		Expression {
			inner: Rc::new (ExpressionData {
				value: value,
				redundant: false,
				source: ExpressionType::Constant,
			}),
		}
	}

	fn add (left: & Expression, right: & Expression) -> Expression {

		let value = left.value () + right.value ();

		let redundant = false
			|| left.redundant () || right.redundant ()
			|| left.includes (value) || right.includes (value);

		let mut total_plus: Vec <Expression> = Vec::new ();
		let mut total_minus: Vec <Expression> = Vec::new ();

		if let ExpressionType::Sum (nest_plus, nest_minus) = & left.inner.source {
			total_plus.extend (nest_plus.iter ().cloned ());
			total_minus.extend (nest_minus.iter ().cloned ());
		} else {
			total_plus.push (left.clone ());
		}

		if let ExpressionType::Sum (nest_plus, nest_minus) = & right.inner.source {
			total_plus.extend (nest_plus.iter ().cloned ());
			total_minus.extend (nest_minus.iter ().cloned ());
		} else {
			total_plus.push (right.clone ());
		}

		total_plus.sort_by_key (|num| Reverse (num.clone ()));
		total_minus.sort_by_key (|num| Reverse (num.clone ()));

		let redundant = redundant || Self::sum_is_redundant (
			value,
			& Iterator::chain (
				total_plus.iter (),
				total_minus.iter (),
			).map (|expr| expr.value ()).collect::<Vec <_>> (),
		);

		Expression {
			inner: Rc::new (ExpressionData {
				value: value,
				redundant: redundant,
				source: ExpressionType::Sum (total_plus, total_minus),
			}),
		}

	}

	fn subtract (left: Expression, right: Expression) -> Expression {

		let value = left.value () - right.value ();

		let redundant = false
			|| left.redundant () || right.redundant ()
			|| left.includes (value) || right.includes (value);

		let mut total_plus: Vec <Expression> = Vec::new ();
		let mut total_minus: Vec <Expression> = Vec::new ();

		if let ExpressionType::Sum (nest_plus, nest_minus) = & left.inner.source {
			total_plus.extend (nest_plus.iter ().cloned ());
			total_minus.extend (nest_minus.iter ().cloned ());
		} else {
			total_plus.push (left.clone ());
		}

		if let ExpressionType::Sum (nest_plus, nest_minus) = & right.inner.source {
			total_plus.extend (nest_minus.iter ().cloned ());
			total_minus.extend (nest_plus.iter ().cloned ());
		} else {
			total_minus.push (right.clone ());
		}

		total_plus.sort_by_key (|num| Reverse (num.clone ()));
		total_minus.sort_by_key (|num| Reverse (num.clone ()));

		let redundant = redundant || Self::sum_is_redundant (
			value,
			& Iterator::chain (
				total_plus.iter (),
				total_minus.iter (),
			).map (|expr| expr.value ()).collect::<Vec <_>> (),
		);

		Expression {
			inner: Rc::new (ExpressionData {
				value: value,
				redundant: redundant,
				source: ExpressionType::Sum (total_plus, total_minus),
			}),
		}

	}

	fn multiply (left: Expression, right: Expression) -> Expression {

		let value = left.value () * right.value ();

		let redundant = false
			|| left.redundant () || right.redundant ()
			|| left.value () == 1 || right.value () == 1
			|| left.includes (value) || right.includes (value);

		let mut total_times: Vec <Expression> = Vec::new ();
		let mut total_divide: Vec <Expression> = Vec::new ();

		if let ExpressionType::Product (times, divide) = & left.inner.source {
			total_times.extend (times.iter ().cloned ());
			total_divide.extend (divide.iter ().cloned ());
		} else {
			total_times.push (left.clone ());
		}

		if let ExpressionType::Product (times, divide) = & right.inner.source {
			total_times.extend (times.iter ().cloned ());
			total_divide.extend (divide.iter ().cloned ());
		} else {
			total_times.push (right.clone ());
		}

		total_times.sort_by_key (|num| Reverse (num.clone ()));
		total_divide.sort_by_key (|num| Reverse (num.clone ()));

		let redundant = redundant || Self::product_is_redundant (
			value,
			& Iterator::chain (
				total_times.iter (),
				total_divide.iter (),
			).map (|expr| expr.value ()).collect::<Vec <_>> (),
		);

		Expression {
			inner: Rc::new (ExpressionData {
				value: value,
				redundant: redundant,
				source: ExpressionType::Product (total_times, total_divide),
			}),
		}

	}

	fn divide (left: Expression, right: Expression) -> Expression {

		let value = left.value () / right.value ();

		let redundant = false
			|| left.redundant () || right.redundant ()
			|| left.value () == 1 || right.value () == 1
			|| left.includes (value) || right.includes (value);

		let mut total_times: Vec <Expression> = Vec::new ();
		let mut total_divide: Vec <Expression> = Vec::new ();

		if let ExpressionType::Product (times, divide) = & left.inner.source {
			total_times.extend (times.iter ().cloned ());
			total_divide.extend (divide.iter ().cloned ());
		} else {
			total_times.push (left.clone ());
		}

		if let ExpressionType::Product (times, divide) = & right.inner.source {
			total_times.extend (divide.iter ().cloned ());
			total_divide.extend (times.iter ().cloned ());
		} else {
			total_divide.push (right.clone ());
		}

		total_times.sort_by_key (|num| Reverse (num.clone ()));
		total_divide.sort_by_key (|num| Reverse (num.clone ()));

		let redundant = redundant || Self::product_is_redundant (
			value,
			& Iterator::chain (
				total_times.iter (),
				total_divide.iter (),
			).map (|expr| expr.value ()).collect::<Vec <_>> (),
		);

		Expression {
			inner: Rc::new (ExpressionData {
				value: value,
				redundant: redundant,
				source: ExpressionType::Product (total_times, total_divide),
			}),
		}

	}

	fn value (& self) -> u64 {
		self.inner.value
	}

	fn redundant (& self) -> bool {
		self.inner.redundant
	}

	fn includes (& self, value: u64) -> bool {

		if self.value () == value {
			return true;
		}

		match & self.inner.source {

			ExpressionType::Constant => return false,

			ExpressionType::Sum (plus, minus) => false
				|| plus.iter ().any (|expr| expr.includes (value))
				|| minus.iter ().any (|expr| expr.includes (value)),

			ExpressionType::Product (times, divide) => false
				|| times.iter ().any (|expr| expr.includes (value))
				|| divide.iter ().any (|expr| expr.includes (value)),

		}

	}

	fn with_parens (& self) -> String {

		match self.inner.source {
			ExpressionType::Constant => format! ("{}", & self),
			_ => format! ("({})", & self),
		}

	}

	fn sum_is_redundant (target: u64, values: & [u64]) -> bool {
		Self::sum_is_redundant_real (target, 0, 0, values, false)
	}

	fn sum_is_redundant_real (
		target: u64,
		plus: u64,
		minus: u64,
		values: & [u64],
		removed: bool,
	) -> bool {

		if values.is_empty () {

			if ! removed {
				return false;
			}

			if plus <= minus {
				return false;
			}

			return plus - minus == target;

		}

		let value = values [0];
		let rest = & values [1 ..];

		return false
			|| Self::sum_is_redundant_real (target, plus + value, minus, rest, removed)
			|| Self::sum_is_redundant_real (target, plus, minus + value, rest, removed)
			|| Self::sum_is_redundant_real (target, plus, minus, rest, true);

	}

	fn product_is_redundant (target: u64, values: & [u64]) -> bool {
		Self::product_is_redundant_real (target, 0, 0, values, false)
	}

	fn product_is_redundant_real (
		target: u64,
		times: u64,
		divide: u64,
		values: & [u64],
		removed: bool,
	) -> bool {

		if values.is_empty () {

			if ! removed {
				return false;
			}

			if divide == 0 || times % divide != 0 {
				return false;
			}

			return times / divide == target;

		}

		let value = values [0];
		let rest = & values [1 ..];

		return false
			|| Self::product_is_redundant_real (target, times * value, divide, rest, removed)
			|| Self::product_is_redundant_real (target, times, divide * value, rest, removed)
			|| Self::product_is_redundant_real (target, times, divide, rest, true);

	}

}

impl fmt::Display for Expression {

	fn fmt (& self, formatter: & mut fmt::Formatter) -> fmt::Result {

		use ExpressionType::*;

		match & self.inner.source {

			Constant => {
				write! (formatter, "{}", self.inner.value) ?;
			},

			Sum (positive, negative) => {

				write! (formatter, "{}", positive [0].with_parens ()) ?;

				for expr in & positive [1 ..] {
					write! (formatter, " + {}", expr.with_parens ()) ?;
				}

				for expr in & negative [..] {
					write! (formatter, " − {}", expr.with_parens ()) ?;
				}

			},

			Product (positive, negative) => {

				write! (formatter, "{}", positive [0].with_parens ()) ?;

				for expr in & positive [1 ..] {
					write! (formatter, " × {}", expr.with_parens ()) ?;
				}

				for expr in & negative [..] {
					write! (formatter, " ÷ {}", expr.with_parens ()) ?;
				}


			},

		};

		Ok (())

	}

}

#[ derive (Eq, Ord, PartialEq, PartialOrd) ]
enum ExpressionType {
	Constant,
	Sum (Vec <Expression>, Vec <Expression>),
	Product (Vec <Expression>, Vec <Expression>),
}

