
module.exports = {
   run: async function (s) {
      let x = import('./pkg').then(r =>
          r.interpret(s)
      );
      let y = await x;
      return y;
   }
};

