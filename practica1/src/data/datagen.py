import csv
import os
import datetime
import random

# Get the last index from the CSV if it exists
last_index = 0
file_exists = os.path.exists('./data.csv') and os.path.getsize('./data.csv') > 0
if file_exists:
    with open('./data.csv', 'r') as csvfile:
        reader = csv.DictReader(csvfile)
        for row in reader:
            if 'Index' in row:
                try:
                    index = int(row['Index'])
                    last_index = max(last_index, index)
                except ValueError:
                    pass
    print(f"Last index found: {last_index}")
else:
    print("No existing CSV file found or it is empty. Starting from index 0.")

# Now open the file for appending or writing
mode = 'a' if file_exists else 'w'
with open('data.csv', mode, newline='') as csvfile:
    fieldnames = ['VendorID','tpep_pickup_datetime','tpep_dropoff_datetime','passenger_count','trip_distance','RatecodeID','store_and_fwd_flag','PULocationID','DOLocationID','payment_type','fare_amount','extra','mta_tax','tip_amount','tolls_amount','improvement_surcharge','total_amount','congestion_surcharge','Index']
    writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
    
    # Write header only if creating a new file
    if not file_exists:
        writer.writeheader()
    
    # Write the data to the CSV in loop
    for i in range(1, 10000000):
        # Create a dictionary for the row
        trip_distance = round(random.uniform(0.5, 10.0), 2)
        fare_amount = (round(random.uniform(3.0, 50.0), 2)*round(random.uniform(0.5, 10.0), 2))/2
        tip = fare_amount * 0.1
        mta_tax = fare_amount * 0.05
        total_amount = fare_amount + tip + mta_tax + 0.5 + 0.3 + 2.5
        pickupDate = datetime.date.today().isoformat()
        pickupDateIso = datetime.datetime.strptime(pickupDate, '%Y-%m-%d').isoformat()
        dropOffDate = datetime.date.today() + datetime.timedelta(days=random.randint(1, 10))
        dropOffDateIso = datetime.datetime.strptime(str(dropOffDate), '%Y-%m-%d').isoformat()
        row = {
            'VendorID': str(1 + i),
            'tpep_pickup_datetime': pickupDateIso,
            'tpep_dropoff_datetime': dropOffDateIso,
            'passenger_count': random.randint(1, 5),
            'trip_distance': trip_distance,
            'RatecodeID': '1',
            'store_and_fwd_flag': 'N',
            'PULocationID': random.randint(1, 100),
            'DOLocationID': random.randint(1, 100),
            'payment_type': random.choice(['1', '2', '3', '4', '5']),
            'fare_amount': fare_amount,
            'extra': '0.5',
            'mta_tax': mta_tax,
            'tip_amount': tip,
            'tolls_amount': '0.0',
            'improvement_surcharge': '0.3',
            'total_amount': total_amount,
            'congestion_surcharge': '2.5',
            # Correct index calculation (was incorrect in original)
            'Index': last_index + i
        }
        # Write the row to the CSV
        writer.writerow(row)
with open('./data.csv', 'r') as csvfile:
    reader = csv.DictReader(csvfile)
    for row in reader:
        if 'Index' in row:
            try:
                index = int(row['Index'])
                last_index = max(last_index, index)
            except ValueError:
                pass
    print(f"New last index: {last_index}")
