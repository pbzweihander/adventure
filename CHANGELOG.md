0.2.0 (not released)
--------------------

 - If both of `futures01` and `std-future` features are enabled, `&Waker` in
   `Response::poll` will be not ignored even if it is polled from futures 0.1
   or goes into them. It will prevent a potential freezing bug.
 - `Response::Waker` associated type is removed.
 - `std-futures` feature is renamed to `std-future`.


0.1.0 (March 22, 2019)
----------------------

 - Initial release.