export default async function Command() {
  process.stderr.write("x".repeat(100000));
  console.log("end-of-bounded-log");
}
