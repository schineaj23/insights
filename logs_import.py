import requests

def main():
    r = requests.get('https://api.github.com/events')
    print(r)

if __name__ == "__main__":
    main()