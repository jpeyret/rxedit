## Sample files to execute rxedit against. 

Don't modify without a reason, they are used for regression testing.

#### How the testing works

- `lazy.py` reads commands from its matching `lazy.yaml` and calls `rxedit` against the sample files

- the `lazy_regression_tests` package is used to check that each command returned the exact same results as previously.

  - this relies on the existence of 2 directories tracked in environment variables

  - `$lzrt_template_dirname_exp` which should be kept stable - tracks past results, which become expectations.

  - `$lzrt_template_dirname_got` which could be kept in a volatile location, tracks the latest results

    - if `$lzrt_template_dirname_exp` starts out empty, the latest results are copied into it.

  - a less important directory `$lzrt_info_dir` is where the results of each test gets saved in json files.

    

This is still of a work in progress in terms of configuring the lazy_regression_tests environment, but once configured it efficiently 
checks that modifications and fixes to rxedit do not impact known-good results.

