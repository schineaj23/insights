import boto3

# Define your AWS region and SQS queue URL
aws_region = 'us-east-1'
queue_url = 'https://sqs.us-east-1.amazonaws.com/098150209510/LogInsertQueue'


# Initialize the SQS client
sqs = boto3.client('sqs', region_name=aws_region)

# Function to send a message to the SQS queue
def send_message_to_sqs(message_body):
    response = sqs.send_message(
        QueueUrl=queue_url,
        MessageBody=message_body
    )
    return response

# File containing numbers separated by commas
file_path = '/home/cat/src/insighttf/insights-services/importer/season_8_logs.csv'

try:
    with open(file_path, 'r') as file:
        # Read and process each line
        for line in file:
            # Split the line into numbers separated by commas
            numbers = line.strip().split(',')
            
            # Send each number as an SQS message
            for number in numbers:
                response = send_message_to_sqs(number)
                print(f"Sent message: {number}, MessageId: {response['MessageId']}")

except FileNotFoundError:
    print(f"File '{file_path}' not found.")
except Exception as e:
    print(f"An error occurred: {str(e)}")
