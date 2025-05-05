#include <iostream>
#include <fstream>
#include <sstream>
#include <vector>
#include <unordered_map>
#include <string>

struct Trip
{
    std::string vendor_id;
    std::string tpep_pickup_datetime;
    std::string tpep_dropoff_datetime;
    std::string passenger_count;
    std::string trip_distance;
    std::string ratecode_id;
    std::string store_and_fwd_flag;
    std::string pu_location_id;
    std::string do_location_id;
    std::string payment_type;
    std::string fare_amount;
    std::string extra;
    std::string mta_tax;
    std::string tip_amount;
    std::string tolls_amount;
    std::string improvement_surcharge;
    std::string total_amount;
    std::string congestion_surcharge;
    std::string index;
};

std::vector<std::string> read_csv(const std::string &filename)
{
    std::ifstream file(filename);
    if (!file.is_open())
    {
        throw std::runtime_error("Could not open file");
    }

    std::vector<std::string> array;
    std::string line;

    std::getline(file, line); // esta tavuel debería leer y no contar los encabezados

    while (std::getline(file, line))
    {
        std::stringstream ss(line);
        std::string field;
        while (std::getline(ss, field, ','))
        {
            array.push_back(field);
        }
    }

    file.close();
    return array;
}

int main()
{
    try
    {
        std::string filename = "src/yellow_tripdata_2020-06.csv";
        std::vector<std::string> array = read_csv(filename);

        std::cout << "Number of cells: " << array.size() << std::endl;

        std::unordered_map<std::string, Trip> trips_map;
        size_t i = 0;
        while (i + 18 < array.size())
        {
            Trip trip = {
                array[i],
                array[i + 1],
                array[i + 2],
                array[i + 3],
                array[i + 4],
                array[i + 5],
                array[i + 6],
                array[i + 7],
                array[i + 8],
                array[i + 9],
                array[i + 10],
                array[i + 11],
                array[i + 12],
                array[i + 13],
                array[i + 14],
                array[i + 15],
                array[i + 16],
                array[i + 17],
                array[i + 18]};
            trips_map[trip.index] = trip;
            i += 19;
        }

        // Pedir el índice por consola
        std::cout << "Enter the index to find: ";
        std::string index_to_find;
        std::getline(std::cin, index_to_find);

        // Buscar el índice en el mapa
        auto it = trips_map.find(index_to_find);
        if (it != trips_map.end())
        {
            const Trip &found_trip = it->second;
            std::cout << "Found trip with vendor_id: " << found_trip.vendor_id
                      << " and pickup time: " << found_trip.tpep_pickup_datetime << std::endl;
        }
        else
        {
            std::cout << "Trip with index " << index_to_find << " not found" << std::endl;
        }
    }
    catch (const std::exception &e)
    {
        std::cerr << "Error: " << e.what() << std::endl;
    }

    return 0;
}