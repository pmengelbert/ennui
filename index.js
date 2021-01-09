
module.exports = {
   run: async function (s) {
      let x = import('./pkg').then(r =>
          r.interpret(s)
      );
      let y = await x;
      return y;
   },
   color: function (s) {
      var Convert = require('ansi-to-html');
      var convert = new Convert();

      return convert.toHtml(s);
   }
};

