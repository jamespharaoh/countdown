use clap::Parser;
use std::iter;

#[ derive (clap::Parser) ]
struct Args {
	#[ command (subcommand) ]
	command: Commands,
}

#[ derive (clap::Subcommand) ]
enum Commands {
	Random,
	Solve (SolveArgs),
}

#[ derive (clap::Args) ]
struct SolveArgs {
	target: u64,
	numbers: Vec <u64>,
}

fn main () {
	match Args::parse ().command {
		Commands::Random => random (),
		Commands::Solve (args) => solve (args),
	}
}

fn random () {

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

	solve_and_print (target, & numbers);

}

fn take_random (available: & mut Vec <u64>) -> u64 {
	let index = rand::random::<usize> () % available.len ();
	available.remove (index)
}

fn solve (args: SolveArgs) {
	solve_and_print (args.target, & args.numbers);
}

fn solve_and_print (target: u64, numbers: & [u64]) {

	let (
		simple_solutions,
		redundant_solutions,
	) = countdown::solve (& numbers, target).into_iter ().partition::<Vec <_>, _> (
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
