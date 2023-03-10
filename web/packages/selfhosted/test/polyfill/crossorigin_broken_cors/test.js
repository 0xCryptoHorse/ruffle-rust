const {
    open_test,
    inject_ruffle_and_wait,
    play_and_monitor,
} = require("../../utils");
const { expect, use } = require("chai");
const chaiHtml = require("chai-html");
const fs = require("fs");

use(chaiHtml);

// [NA] Disabled for now as the test can take too long on CI
describe.skip("Doesn't error with cross-origin frames", () => {
    it("Loads the test", async () => {
        await open_test(browser, __dirname);
    });

    it("Polyfills with ruffle", async () => {
        await inject_ruffle_and_wait(browser);
        const actual = await browser.$("#test-container").getHTML(false);
        const expected = fs.readFileSync(`${__dirname}/expected.html`, "utf8");
        expect(actual).html.to.equal(expected);
    });

    it("Plays a movie", async () => {
        await play_and_monitor(
            browser,
            await browser.$("#test-container").$("<ruffle-embed />")
        );
    });
});
