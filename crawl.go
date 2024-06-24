package main

import (
    "bufio"
    "fmt"
    "os"
    "time"
    "strconv"
    "errors"

	"github.com/gocolly/colly/v2"
)


type Result struct {
	Code    string
	Url     string
}

func getConcurrency() int {
    value, exists := os.LookupEnv("CONCURRENCY")
    if !exists {
        return 20
    }
    intVal, err := strconv.Atoi(value)
    if err != nil {
        return 20
    }
    return intVal
}

func main() {
    CONCURRENCY := getConcurrency()
    results := []Result{}

    fmt.Printf("Concurrency: %d\n", CONCURRENCY)


	// Instantiate default collector
	c := colly.NewCollector(
	    colly.MaxDepth(0),
	    colly.Async(),
	    colly.UserAgent("crawler-test"),
	    colly.IgnoreRobotsTxt(),
    )

	c.Limit(&colly.LimitRule{DomainGlob: "*", Parallelism: CONCURRENCY})

	// Before making a request put the URL with
	// the key of "url" into the context of the request
	c.OnRequest(func(r *colly.Request) {
		r.Ctx.Put("url", r.URL.String())
	})

	// After making a request get "url" from
	// the context of the request
	c.OnResponse(func(r *colly.Response) {
	    code := strconv.Itoa(r.StatusCode)
        results = append(results, Result{ Code: code, Url: r.Ctx.Get("url")})
		fmt.Println(r.Ctx.Get("url"), code)
	})

	// Set error handler
	c.OnError(func(r *colly.Response, err error) {
	    code := strconv.Itoa(r.StatusCode)
	    if code == "0" {
	        urlError := errors.Unwrap(err)
            code = fmt.Sprintf("%T", urlError)
//             fmt.Printf("***** %#v\n", urlError)
	    }
        results = append(results, Result{ Code: code, Url: r.Request.URL.String()})
        fmt.Println(r.Request.URL, code)
    })

    file, err := os.Open("../jobanni/data/top-1000.txt")
    if err != nil {
        panic(err)
    }
    defer file.Close()

    var urls []string
    scanner := bufio.NewScanner(file)
    for scanner.Scan() {
        urls = append(urls, scanner.Text())
    }

    start := time.Now()

    for _, url := range urls {
        c.Visit(url)
    }

	// Wait until threads are finished
	c.Wait()

    aggregates := make(map[string]int)

    for _, result := range results {
        aggregates[result.Code]++
    }

    fmt.Printf("Total time: %.2fs\n", time.Since(start).Seconds())

    for code, count := range aggregates {
        fmt.Printf("%s: %d\n", code, count)
    }
}