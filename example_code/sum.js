const readline = require("readline");

const rl = readline.createInterface({
  input: process.stdin,
});

const numbers = [];
rl.on("line", (line) => {
  numbers.push(parseInt(line));
  if (numbers.length === 3) {
    const sum = numbers.reduce((a, b) => a + b, 0);
    console.log(sum);
    rl.close();
    process.exit(0);
  }
});
