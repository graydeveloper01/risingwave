
INSERT INTO
  orders (
    order_id,
    order_date,
    customer_name,
    price,
    product_id,
    order_status
  )
VALUES
  (11, 1558430840000, 'Bob', 10.50, 1, 1),
  (12, 1558430840001, 'Alice', 20.50, 2, 1),
  (13, 1558430840002, 'Alice', 18.50, 2, 1);


INSERT INTO single_type VALUES (13, '23:59:59.999')


INSERT INTO sqlserver_all_data_types VALUES (11, 'False', 0, 0, 0, 0, 0, 0, 0, '', NULL, '2001-01-01', '00:00:00', '2001-01-01 00:00:00', '2001-01-01 00:00:00');

INSERT INTO sqlserver_all_data_types VALUES (12, 'True', 255, -32768, -2147483648, -9223372036854775808, -10.0, -9999.999999, -10000.0, 'aa', 0xff, '1990-01-01', '13:59:59.123', '2000-01-01 11:00:00.123', '1990-01-01 00:00:01.123');

INSERT INTO sqlserver_all_data_types VALUES (13, 'True', 127, 32767, 2147483647, 9223372036854775807, -10.0, 9999.999999, 10000.0, 'zzzz', 0xffffffff, '2999-12-31', '23:59:59.999', '2099-12-31 23:59:59.999', '2999-12-31 23:59:59.999')
