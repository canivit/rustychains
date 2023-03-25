const readline = require("readline");

const rl = readline.createInterface({
  input: process.stdin,
});

rl.on("line", (line) => {
  const point = JSON.parse(line);
  point.x += 7;
  point.y += 4;
  console.log(JSON.stringify(point));
  rl.close();
  process.exit(0);
});
